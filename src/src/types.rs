use std::{
    collections::HashMap, path::PathBuf, sync::{Arc, Mutex}
};
use tokio::sync::mpsc;

use uuid::Uuid;
use crate::wayland::Output;
use wayland_client::{protocol::wl_seat, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};

#[derive(Debug)]
pub enum Request {
    ReloadConfig,
    RunCommand(String),
    DbEvent(String), 
    OnBattery(bool),
    Flush,
    Inhibit,
}

pub type NotificationListHandle =
    Arc<Mutex<HashMap<Uuid, (String, Option<String>, bool, ext_idle_notification_v1::ExtIdleNotificationV1)>>>;

#[derive(Debug, Default)]
pub struct WaylandGlobals {
    pub seat: Option<wl_seat::WlSeat>,
    pub notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    pub on_battery: Option<bool>,
    pub restore_cmd: Option<String>,
    pub is_paused: bool,
}
pub type SharedGlobals = Arc<Mutex<WaylandGlobals>>;

#[derive(Debug)]
pub struct State {
    pub(crate) globals: SharedGlobals,
    pub(crate) wl_seat: Option<wl_seat::WlSeat>,
    pub(crate) qh: QueueHandle<State>,
    pub(crate) idle_notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    pub(crate) notification_list: NotificationListHandle,
    pub(crate) tx: mpsc::Sender<Request>,
    pub(crate) config_path: PathBuf,
    pub(crate) outputs: HashMap<u32, Output>,
}