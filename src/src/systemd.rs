use std::fs;
use log::{info, error};
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
}

/// Helper to get the proxy connection to the Session Bus (User Systemd)
async fn get_manager() -> Result<SystemdManagerProxy<'static>> {
    let connection = Connection::session().await?;
    SystemdManagerProxy::new(&connection).await
}

pub async fn start() -> anyhow::Result<()> {
    let manager = get_manager().await?;
    info!("Starting {}...", SERVICE_NAME);
    // Mode "replace" will start it if stopped, or restart if running (conceptually)
    // usually we use "replace" for start
    manager.start_unit(SERVICE_NAME, "replace").await?;
    info!("Service started successfully.");
    Ok(())
}

pub async fn stop() -> anyhow::Result<()> {
    let manager = get_manager().await?;
    info!("Stopping {}...", SERVICE_NAME);
    manager.stop_unit(SERVICE_NAME, "replace").await?;
    info!("Service stopped.");
    Ok(())
}

pub async fn restart() -> anyhow::Result<()> {
    let manager = get_manager().await?;
    info!("Restarting {}...", SERVICE_NAME);
    manager.restart_unit(SERVICE_NAME, "replace").await?;
    info!("Service restarted.");
    Ok(())
}

pub async fn install() -> anyhow::Result<()> {
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

    Ok(())
}

pub async fn enable() -> anyhow::Result<()> {
    let manager = get_manager().await?;
    match manager.enable_unit_files(&[SERVICE_NAME], false, true).await {
        Ok(_) => info!("Service enabled."),
        Err(e) => error!("Failed to enable service: {}", e),
    }
    manager.reload().await?;

    Ok(())
}

pub async fn disable() -> anyhow::Result<()> {
    let manager = get_manager().await?;
    match manager.disable_unit_files(&[SERVICE_NAME], false).await {
        Ok(_) => info!("Service disabled."),
        Err(e) => error!("Failed to disable service: {}", e),
    }
    manager.reload().await?;

    Ok(())
}