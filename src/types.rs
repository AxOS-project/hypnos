use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

use uuid::Uuid;
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
    Arc<Mutex<HashMap<Uuid, (String, ext_idle_notification_v1::ExtIdleNotificationV1)>>>;

#[derive(Debug)]
pub struct State {
    pub(crate) wl_seat: Option<wl_seat::WlSeat>,
    pub(crate) qh: QueueHandle<State>,
    pub(crate) idle_notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    pub(crate) notification_list: NotificationListHandle,
    pub(crate) tx: mpsc::Sender<Request>,
    pub(crate) config_path: PathBuf,
}