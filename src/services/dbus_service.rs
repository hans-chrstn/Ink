use mlua::{Function, Lua, Table};
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::sync::{
    Arc, Mutex,
    mpsc::{Sender, channel},
};
use zbus::zvariant::Value;
use zbus::{connection, interface};

#[derive(Debug)]
pub enum DbusUpdate {
    Notification(Notification),
    Tray(TrayUpdate),
}

#[derive(Debug)]
pub struct Notification {
    app_name: String,
    summary: String,
    body: String,
    timeout: i32,
}

#[derive(Debug)]
pub enum TrayUpdate {
    ItemRegistered(String),
}

struct NotificationServer {
    sender: Sender<DbusUpdate>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "summary".to_string()]
    }

    async fn notify(
        &mut self,
        app_name: String,
        _replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let notif = Notification {
            app_name,
            summary,
            body,
            timeout: expire_timeout,
        };
        self.sender.send(DbusUpdate::Notification(notif)).unwrap();
        1
    }

    async fn close_notification(&self, _id: u32) {}

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Ink".to_string(),
            "ink-org".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

struct StatusNotifierWatcher {
    sender: Sender<DbusUpdate>,
    items: Arc<Mutex<Vec<String>>>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(&mut self, service: &str) {
        let mut items = self.items.lock().unwrap();
        if !items.contains(&service.to_string()) {
            items.push(service.to_string());
            self.sender
                .send(DbusUpdate::Tray(TrayUpdate::ItemRegistered(
                    service.to_string(),
                )))
                .unwrap();
        }
    }

    async fn register_status_notifier_host(&mut self, _service: &str) {}

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().unwrap().clone()
    }
    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}

pub fn init(lua: Rc<Lua>) {
    let (sender, receiver) = channel();

    glib::idle_add_local({
        let lua = lua.clone();
        move || {
            if let Ok(update) = receiver.try_recv() {
                match update {
                    DbusUpdate::Notification(n) => {
                        if let Err(e) = handle_notification_in_lua(&lua, n) {
                            eprintln!("Error handling notification in Lua: {}", e);
                        }
                    }
                    DbusUpdate::Tray(t) => {
                        if let Err(e) = handle_tray_update_in_lua(&lua, t) {
                            eprintln!("Error handling tray update in Lua: {}", e);
                        }
                    }
                }
            }
            glib::ControlFlow::Continue
        }
    });

    glib::MainContext::default().spawn_local(async {
        if let Err(e) = run_server(sender).await {
            eprintln!("Failed to start DBus server: {}", e);
        }
    });
}

async fn run_server(sender: Sender<DbusUpdate>) -> Result<(), Box<dyn Error>> {
    let notif_server = NotificationServer {
        sender: sender.clone(),
    };
    let tray_server = StatusNotifierWatcher {
        sender,
        items: Arc::new(Mutex::new(Vec::new())),
    };

    let conn = connection::Builder::session()?
        .serve_at("/org/freedesktop/Notifications", notif_server)?
        .serve_at("/StatusNotifierWatcher", tray_server)?
        .build()
        .await?;

    conn.request_name("org.freedesktop.Notifications").await?;
    conn.request_name("org.kde.StatusNotifierWatcher").await?;

    std::future::pending::<()>().await;
    Ok(())
}

fn handle_notification_in_lua(lua: &Lua, notification: Notification) -> mlua::Result<()> {
    let globals = lua.globals();
    let app_table: Table = globals.get("app")?;
    if let Ok(on_notification) = app_table.get::<Function>("on_notification") {
        let params = lua.create_table()?;
        params.set("app_name", notification.app_name.as_str())?;
        params.set("summary", notification.summary.as_str())?;
        params.set("body", notification.body.as_str())?;
        params.set("timeout", notification.timeout)?;
        on_notification.call::<()>(params)?;
    }
    Ok(())
}

fn handle_tray_update_in_lua(lua: &Lua, update: TrayUpdate) -> mlua::Result<()> {
    let globals = lua.globals();
    let app_table: Table = globals.get("app")?;
    if let Ok(tray_table) = app_table.get::<Table>("tray") {
        match update {
            TrayUpdate::ItemRegistered(service) => {
                if let Ok(callback) = tray_table.get::<Function>("on_item_added") {
                    callback.call::<()>(service)?;
                }
            }
        }
    }
    Ok(())
}
