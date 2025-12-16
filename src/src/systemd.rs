use std::fs;
use log::{error, info, warn};
use zbus::{proxy, Connection, Result};
use zbus::zvariant::OwnedObjectPath;

const SERVICE_TEMPLATE: &str = include_str!("../res/hypnos.service.in");
const SERVICE_NAME: &str = "hypnos.service";

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManager {
    fn start_unit(&self, name: &str, mode: &str) -> Result<OwnedObjectPath>;
    fn stop_unit(&self, name: &str, mode: &str) -> Result<OwnedObjectPath>;
    fn restart_unit(&self, name: &str, mode: &str) -> Result<OwnedObjectPath>;
    fn reload(&self) -> Result<()>;
    // Signature: (asbb) -> (ba(sss))
    fn enable_unit_files(&self, files: &[&str], runtime: bool, force: bool) -> Result<(bool, Vec<(String, String, String)>)>;
    fn disable_unit_files(&self, files: &[&str], runtime: bool) -> Result<Vec<(String, String, String)>>;
    fn get_unit_file_state(&self, file: &str) -> Result<String>;
    fn get_unit(&self, name: &str) -> Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
trait Unit {
    #[zbus(property)]
    fn active_state(&self) -> Result<String>;
}

/// Helper to get the proxy connection to the Session Bus (User Systemd)
async fn get_manager() -> Result<SystemdManagerProxy<'static>> {
    let connection = Connection::session().await?;
    SystemdManagerProxy::new(&connection).await
}

fn check_service_installed() -> bool {
    let xdg_dirs = xdg::BaseDirectories::new();
    if let Some(config_home) = xdg_dirs.get_config_home() {
        let service_path = config_home.join("systemd/user").join(SERVICE_NAME);
        service_path.exists()
    } else {
        false
    }
}

async fn is_running() -> anyhow::Result<bool> {
    let connection = Connection::session().await?;
    let manager = SystemdManagerProxy::new(&connection).await?;
    
    let unit_path = manager.get_unit(SERVICE_NAME).await?;
    
    let unit = UnitProxy::builder(&connection)
        .path(unit_path)?
        .build()
        .await?;
        
    let state = unit.active_state().await?;
    
    // Common states: "active", "reloading", "inactive", "failed", "activating", "deactivating"
    Ok(state == "active")
}

pub async fn is_enabled() -> anyhow::Result<bool> {
    let manager = get_manager().await?;
    let state = manager.get_unit_file_state(SERVICE_NAME).await?;
    
    // States that imply enabled: "enabled", "enabled-runtime", "linked", "linked-runtime", "static"
    // For a simple check, "enabled" is usually sufficient for installed services.
    Ok(state == "enabled")
}

pub async fn start() -> anyhow::Result<()> {
    let manager = get_manager().await?;

    if !check_service_installed() {
        warn!("Service is not installed. Installing now...");
        install().await?;
    }

    if is_running().await? {
        info!("Service is already running.");
        return Ok(());
    }

    info!("Starting {}...", SERVICE_NAME);
    // Mode "replace" will start it if stopped, or restart if running (conceptually)
    // usually we use "replace" for start
    manager.start_unit(SERVICE_NAME, "replace").await?;
    info!("Service started successfully.");
    Ok(())
}

pub async fn stop() -> anyhow::Result<()> {
    let manager = get_manager().await?;

    if !is_running().await? {
        info!("Service is not running.");
        return Ok(());
    }

    info!("Stopping {}...", SERVICE_NAME);
    manager.stop_unit(SERVICE_NAME, "replace").await?;
    info!("Service stopped.");
    Ok(())
}

pub async fn restart() -> anyhow::Result<()> {
    let manager = get_manager().await?;

    if !check_service_installed() {
        warn!("Service is not installed. Installing now...");
        install().await?;
    }

    info!("Restarting {}...", SERVICE_NAME);
    manager.restart_unit(SERVICE_NAME, "replace").await?;
    info!("Service restarted.");
    Ok(())
}

pub async fn install() -> anyhow::Result<()> {

    if check_service_installed() {
        info!("Service {} already installed...", SERVICE_NAME);
        return Ok(());
    } else {

        let xdg_dirs = xdg::BaseDirectories::new();
        
        let config_home = xdg_dirs.get_config_home().unwrap();
        let systemd_dir = config_home.join("systemd/user");
        fs::create_dir_all(&systemd_dir)?;
        let service_path = systemd_dir.join(SERVICE_NAME);

        let current_exe = std::env::current_exe()?;
        let exe_str = current_exe.to_str().unwrap_or("hypnos");

        let content = SERVICE_TEMPLATE.replace("@BIN_PATH@", exe_str);
        let home_dir = std::env::var("HOME").unwrap();
        let content = content.replace("@HOME@", &home_dir);

        fs::write(&service_path, content)?;
        info!("Wrote service file to {:?}", service_path);

        let manager = get_manager().await?;
        manager.reload().await?;
        info!("Systemd daemon reloaded.");

        return Ok(());
    }
}

pub async fn enable() -> anyhow::Result<()> {
    let manager = get_manager().await?;

    if !check_service_installed() {
        warn!("Service is not installed. Installing now...");
        install().await?;
    }

    if !is_enabled().await? {
        match manager.enable_unit_files(&[SERVICE_NAME], false, true).await {
            Ok(_) => info!("Service enabled."),
            Err(e) => error!("Failed to enable service: {}", e),
        }
        manager.reload().await?;
        return Ok(());
    } else {
        info!("Service {} is already enabled.", SERVICE_NAME);
        return Ok(());
    }
}

pub async fn disable() -> anyhow::Result<()> {
    let manager = get_manager().await?;

    if !check_service_installed() {
        warn!("Service is not installed. Installing now...");
        install().await?;
    }

    if is_enabled().await? {
        match manager.disable_unit_files(&[SERVICE_NAME], false).await {
            Ok(_) => info!("Service disabled."),
            Err(e) => error!("Failed to disable service: {}", e),
        }
        manager.reload().await?;
        return Ok(());
    } else {
        info!("Service {} is already disabled.", SERVICE_NAME);
        return Ok(());
    }
}