use clap::Parser;
use env_logger::{Builder, Env};
use inotify::{EventMask, Inotify, WatchMask};
use log::{debug, error, info};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{process::Command, sync::mpsc, task::JoinHandle, time::sleep};
use uuid::Uuid;
use wayland::NotificationContext;
use wayland_client::{
    protocol::{wl_surface::WlSurface},
    Connection, EventQueue, QueueHandle,
};
use wayland_protocols::{
    wp::idle_inhibit::zv1::client::{
        zwp_idle_inhibit_manager_v1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
    },
};

use crate::types::{NotificationListHandle, SharedGlobals, WaylandGlobals};

mod config;
mod dbus;
mod joystick_handler;
// mod sunset;
mod types;
mod udev_handler;
mod utils;
mod wayland;

use types::{Request, State};
use udev_handler::UdevHandler;

lazy_static::lazy_static! {
    pub static ref INHIBIT_MANAGER: std::sync::Mutex<Option<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1>> = std::sync::Mutex::new(None);
    pub static ref SURFACE: std::sync::Mutex<Option<WlSurface>> = std::sync::Mutex::new(None);
}
static IS_INHIBITED: AtomicBool = AtomicBool::new(false);

fn ensure_config_file_exists(filename: &str) -> std::io::Result<()> {
    let config_path = utils::xdg_config_path(Some(filename.to_string()))?;
    if !config_path.exists() {
        let mut file = File::create(&config_path)?;
        file.write_all(config::CONFIG_FILE.as_bytes())?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Clone)]
struct IdleRule {
    timeout: i32,
    actions: String,
    #[serde(default)]
    on_battery: Option<bool>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

fn generate_uuid() -> uuid::Uuid {
    Uuid::new_v4()
}

fn load_json_config(path: &Path) -> anyhow::Result<Vec<IdleRule>> {
    let content = fs::read_to_string(path)?;
    let rules: Vec<IdleRule> = serde_json::from_str(&content)?;
    Ok(rules)
}

pub fn apply_config(
    globals: &SharedGlobals,
    qh: &QueueHandle<State>,
    list: &NotificationListHandle,
    config_path: &Path,
) -> anyhow::Result<()> {
    let rules = match load_json_config(config_path) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse JSON config: {}", e);
            return Ok(());
        }
    };

    let global_lock = globals.lock().unwrap();
    if global_lock.notifier.is_none() || global_lock.seat.is_none() {   
        debug!("Cannot apply config yet: notifier or seat missing");
        return Ok(());
    }

    let idle_notifier = global_lock.notifier.as_ref().unwrap();
    let wl_seat = global_lock.seat.as_ref().unwrap();

    let mut map = list.lock().unwrap();
    
    // Cleanup
    for (_, (_, _, notification)) in map.iter() {
        notification.destroy();
    }
    map.clear();

    for rule in rules {
        let ctx = NotificationContext {
            uuid: generate_uuid(),
        };
        debug!("Registering rule: {}s -> '{}' (on_battery: {:?})", rule.timeout, rule.actions, rule.on_battery);

        let notification = idle_notifier.get_idle_notification(
            (rule.timeout * 1000).try_into().unwrap(),
            wl_seat,
            qh,
            ctx.clone(),
        );

        map.insert(ctx.uuid, (rule.actions, rule.on_battery.unwrap_or(false), notification));
    }

    info!("Configuration applied with {} rules", map.len());
    Ok(())
}

async fn run_command(cmd: String) {
    let (cmd_prog, args) = utils::get_args(cmd.clone());
    debug!("Executing: {}", cmd);
    
    tokio::spawn(async move {
        match Command::new(&cmd_prog)
            .args(args)
            .spawn() 
        {
            Ok(mut child) => { 
                match child.wait().await {
                    Ok(status) => debug!("Command '{}' finished with {}", cmd_prog, status),
                    Err(e) => error!("Command '{}' failed to wait: {}", cmd_prog, e),
                }
            }
            Err(e) => error!("Failed to spawn '{}': {}", cmd_prog, e),
        }
    });
}

pub async fn filewatcher_run(config_path: &Path, tx: mpsc::Sender<Request>) -> anyhow::Result<()> {
    let mut inotify = Inotify::init().expect("Error while initializing inotify");
    debug!("Watching {:?}", config_path);
    inotify.watches().add(config_path, WatchMask::MODIFY).expect("Failed to add watch");

    let mut buffer = [0; 1024];
    tokio::task::spawn_blocking(move || loop {
        let events = inotify.read_events_blocking(&mut buffer).expect("Failed to read inotify events");
        for event in events {
            if event.mask.contains(EventMask::MODIFY) && !event.mask.contains(EventMask::ISDIR) {
                debug!("File modified: {:?}", event.name);
                tx.blocking_send(Request::ReloadConfig).unwrap();
            }
        }
    });
    Ok(())
}

#[derive(Clone)]
pub struct WaylandRunner {
    connection: Connection,
    qhandle: QueueHandle<State>,
    tx: mpsc::Sender<Request>,
    notification_list: NotificationListHandle,
    config_path: PathBuf,
    globals: SharedGlobals,
}

impl WaylandRunner {
    pub fn new(
        connection: Connection,
        qhandle: QueueHandle<State>,
        tx: mpsc::Sender<Request>,
        config_path: PathBuf,
        globals: SharedGlobals,
    ) -> Self {
        let map = HashMap::new();
        let notification_list = Arc::new(Mutex::new(map));

        Self {
            connection,
            qhandle,
            tx,
            notification_list,
            config_path,
            globals,
        }
    }

    pub async fn wayland_run(
        &self,
        mut event_queue: EventQueue<State>,
    ) -> anyhow::Result<JoinHandle<Result<(), anyhow::Error>>> {

        let display = self.connection.display();
        display.get_registry(&self.qhandle, ());

        let mut state = State {
            globals: self.globals.clone(),
            wl_seat: None,
            idle_notifier: None,
            qh: self.qhandle.clone(),
            notification_list: self.notification_list.clone(),
            tx: self.tx.clone(),
            outputs: HashMap::new(),
            config_path: self.config_path.clone(),
        };

        Ok(tokio::task::spawn_blocking(move || loop {
            event_queue.blocking_dispatch(&mut state)?;
        }))
    }

    pub async fn process_command(&self, rx: &mut mpsc::Receiver<Request>) -> anyhow::Result<()> {
        while let Some(event) = rx.recv().await {
            match event {
                Request::ReloadConfig => {
                    debug!("Config reload requested");
                    let _ = apply_config(
                        &self.globals,
                        &self.qhandle,
                        &self.notification_list,
                        &self.config_path,
                    );

                    // Little flush to ensure requests are processed
                    let _ = self.connection.flush();
                }
                Request::RunCommand(cmd) => {
                    run_command(cmd).await;
                }
                Request::DbEvent(event_name) => {
                    debug!("DBus event received: {}", event_name);
                }
                Request::OnBattery(state) => {
                    debug!("On Battery: {}", state);
                    self.globals.lock().unwrap().on_battery = Some(state);
                }
                Request::Inhibit => {
                    let _ = self.inhibit_sleep();
                }
                Request::Flush => {
                    let _ = self.connection.flush();
                }
            }
        }
        Ok(())
    }

    fn inhibit_sleep(&self) -> anyhow::Result<()> {
        let qh = self.qhandle.clone();
        let connection = self.connection.clone();
        
        tokio::spawn(async move {
            if IS_INHIBITED.load(Ordering::SeqCst) { return; }
            debug!("Inhibiting sleep");
            IS_INHIBITED.store(true, Ordering::SeqCst);

            let mut inhibitor: Option<ZwpIdleInhibitorV1> = None;
            if let Some(manager) = INHIBIT_MANAGER.lock().unwrap().as_ref() {
                let surface = SURFACE.lock().unwrap();
                if let Some(surface) = surface.as_ref() {
                    inhibitor = Some(manager.create_inhibitor(surface, &qh.clone(), ()));
                    let _ = connection.flush();
                }
            }
            sleep(Duration::from_secs(config::TIMEOUT_SEC)).await;

            if let Some(inhibitor) = inhibitor {
                inhibitor.destroy();
                let _ = connection.flush();
            }
            IS_INHIBITED.store(false, Ordering::SeqCst);
        });
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    
    let _ = ensure_config_file_exists("config.json");

    let (tx, mut rx) = mpsc::channel(32);

    let config_path = utils::xdg_config_path(Some(args.config.clone()))?;
    
    filewatcher_run(&config_path, tx.clone()).await?;

    let connection = Connection::connect_to_env().unwrap();
    let event_queue: EventQueue<State> = connection.new_event_queue();
    let qhandle = event_queue.handle();

    let globals = Arc::new(Mutex::new(WaylandGlobals::default()));

    let wayland_runner = WaylandRunner::new(
        connection.clone(),
        qhandle.clone(),
        tx.clone(),
        config_path,
        globals,
    );

    let udev_handler = UdevHandler::new(tx.clone());

    let _ = wayland_runner.wayland_run(event_queue).await;

    tokio::try_join!(
        dbus::upower_watcher(tx.clone()),
        dbus::logind_watcher(tx.clone()),
        wayland_runner.process_command(&mut rx),
        udev_handler.monitor()
    )?;

    Ok(())
}