use log::{debug, info};
use uuid::Uuid;
use wayland_client::{
    protocol::{
        wl_compositor, wl_output, wl_registry, wl_seat,
        wl_surface::{self},
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::{
    ext::idle_notify::v1::client::{ext_idle_notification_v1, ext_idle_notifier_v1},
    wp::idle_inhibit::zv1::client::{
        zwp_idle_inhibit_manager_v1,
        zwp_idle_inhibitor_v1::{self},
    },
    xdg::activation::v1::client::{xdg_activation_token_v1, xdg_activation_v1},
};

use crate::{apply_config, types::{State, Request}, INHIBIT_MANAGER, SURFACE};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Output {
    reg_name: u32,
    wl_output: wl_output::WlOutput,
    name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NotificationContext {
    pub uuid: Uuid,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_seat" => {
                    let wl_seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                    state.wl_seat = Some(wl_seat.clone());
                    state.globals.lock().unwrap().seat = Some(wl_seat.clone());
                    debug!("wl_seat: {:?}", name);
                    if state.idle_notifier.is_some() {
                         let _ = apply_config(
                            &state.globals,
                            &state.qh,
                            &state.notification_list,
                            &state.config_path
                         );
                    }
                }
                "ext_idle_notifier_v1" => {
                    let idle_notifier = registry
                        .bind::<ext_idle_notifier_v1::ExtIdleNotifierV1, _, _>(name, 1, qh, ());

                    debug!("ext_idle_notifier_v1: {:?}", name);
                    state.idle_notifier = Some(idle_notifier.clone());
                    state.globals.lock().unwrap().notifier = Some(idle_notifier.clone());
                    if state.wl_seat.is_some() {
                        let _ = apply_config(
                            &state.globals,
                            &state.qh,
                            &state.notification_list,
                            &state.config_path
                        );
                    }
                }
                "xdg_activation_v1" => {
                    let _activation = registry.bind::<xdg_activation_v1::XdgActivationV1, _, _>(name, 1, qh, ());
                }
                "xdg_activation_token_v1" => {
                    let _activation = registry.bind::<xdg_activation_token_v1::XdgActivationTokenV1, _, _>(name, 1, qh, ());
                }
                "zwp_idle_inhibitor_v1" => {
                    let _inhibitor = registry.bind::<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, _, _>(name, 1, qh, ());
                }
                "zwp_idle_inhibit_manager_v1" => {
                    let inhibit_manager = registry.bind::<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, _, _>(name, 1, qh, ());
                    *INHIBIT_MANAGER.lock().unwrap() = Some(inhibit_manager);
                }
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    *SURFACE.lock().unwrap() = Some(surface);
                }
                "wl_output" => {
                    let wl_output = registry.bind::<wl_output::WlOutput, _, _>(name, 1, qh, ());
                    let output = Output {
                        reg_name: name,
                        wl_output,
                        name: None,
                    };
                    state.outputs.insert(name, output);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, ()> for State {
    fn event(_: &mut Self, _: &zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, _: zwp_idle_inhibitor_v1::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, ()> for State {
    fn event(_: &mut Self, _: &zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, _: zwp_idle_inhibit_manager_v1::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<xdg_activation_v1::XdgActivationV1, ()> for State {
    fn event(_: &mut Self, _: &xdg_activation_v1::XdgActivationV1, _: xdg_activation_v1::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<xdg_activation_token_v1::XdgActivationTokenV1, ()> for State {
    fn event(_: &mut Self, _: &xdg_activation_token_v1::XdgActivationTokenV1, _: xdg_activation_token_v1::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(_: &mut Self, _: &wl_compositor::WlCompositor, _: wl_compositor::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(_: &mut Self, _: &wl_surface::WlSurface, _: wl_surface::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<wl_output::WlOutput, ()> for State {
     fn event(_state: &mut Self, _output: &wl_output::WlOutput, _event: wl_output::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}
impl Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for State {
    fn event(_: &mut Self, _: &ext_idle_notifier_v1::ExtIdleNotifierV1, _: ext_idle_notifier_v1::Event, _: &(), _: &Connection, _qh: &QueueHandle<Self>) {}
}

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, NotificationContext> for State {
    fn event(
        state: &mut Self,
        _idle_notification: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        ctx: &NotificationContext,
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        debug!("Idle Notification event: {:?} uuid: {:?}", event, ctx.uuid);

        if state.globals.lock().unwrap().is_paused {
            debug!("System is paused, ignoring idle notification event");
            return;
        }
        
        match event {
            ext_idle_notification_v1::Event::Idled => {
                let map = state.notification_list.lock().unwrap();

                if let Some((command, _, req_battery, _)) = map.get(&ctx.uuid) {

                    let current_bat_state = state.globals.lock().unwrap().on_battery;

                    if *req_battery && !current_bat_state.unwrap_or(false) {
                        debug!("Skipping idle command '{}' because on AC power", command);
                        return;
                    }

                    info!("Idle reached, executing: {}", command);
                    let _ = state.tx.try_send(Request::RunCommand(command.clone()));
                }
            }
            ext_idle_notification_v1::Event::Resumed => {
                let map = state.notification_list.lock().unwrap();

                if let Some((_, Some(restore_cmd), _, _)) = map.get(&ctx.uuid) {
                    info!("Idle resumed, executing restore command: {}", restore_cmd);
                    let _ = state.tx.try_send(Request::RunCommand(restore_cmd.clone()));
                } else {
                    debug!("No restore command defined for uuid {:?}", ctx.uuid);
                }
            }
            _ => {}
        }
    }
}