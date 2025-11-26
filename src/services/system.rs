use mlua::{Lua, Result as LuaResult, Value};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::process::Command;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum SystemError {
    IoError(io::Error),
    CommandError(String),
    ParseError(String),
    Utf8Error(FromUtf8Error),
    NotFound(String),
    Other(String),
}

impl Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::IoError(e) => write!(f, "I/O error: {}", e),
            SystemError::CommandError(e) => write!(f, "Command execution error: {}", e),
            SystemError::ParseError(e) => write!(f, "Parsing error: {}", e),
            SystemError::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
            SystemError::NotFound(e) => write!(f, "Item not found: {}", e),
            SystemError::Other(e) => write!(f, "System error: {}", e),
        }
    }
}

impl std::error::Error for SystemError {}

impl From<io::Error> for SystemError {
    fn from(err: io::Error) -> Self {
        SystemError::IoError(err)
    }
}

impl From<FromUtf8Error> for SystemError {
    fn from(err: FromUtf8Error) -> Self {
        SystemError::Utf8Error(err)
    }
}

impl From<SystemError> for mlua::Error {
    fn from(err: SystemError) -> Self {
        mlua::Error::external(err)
    }
}

pub trait BatteryService {
    fn get_capacity(&self) -> std::result::Result<i32, SystemError>;
    fn get_status(&self) -> std::result::Result<String, SystemError>;
}

pub struct LinuxBatteryService;

impl BatteryService for LinuxBatteryService {
    fn get_capacity(&self) -> std::result::Result<i32, SystemError> {
        let battery_dir = fs::read_dir("/sys/class/power_supply/")?
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_string_lossy().starts_with("BAT"))
            .ok_or_else(|| SystemError::NotFound("No battery found".into()))?
            .path();

        fs::read_to_string(battery_dir.join("capacity"))?
            .trim()
            .parse::<i32>()
            .map_err(|e| SystemError::ParseError(format!("Failed to parse capacity: {}", e)))
    }

    fn get_status(&self) -> std::result::Result<String, SystemError> {
        let battery_dir = fs::read_dir("/sys/class/power_supply/")?
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_string_lossy().starts_with("BAT"))
            .ok_or_else(|| SystemError::NotFound("No battery found".into()))?
            .path();

        fs::read_to_string(battery_dir.join("status"))
            .map(|s| s.trim().to_string())
            .map_err(SystemError::IoError)
    }
}

pub trait WifiService {
    fn get_ssid(&self) -> std::result::Result<String, SystemError>;
}

#[derive(Clone)]
pub struct LinuxWifiService;

impl WifiService for LinuxWifiService {
    fn get_ssid(&self) -> std::result::Result<String, SystemError> {
        let output = Command::new("nmcli")
            .args(["-t", "-f", "active,ssid", "dev", "wifi"])
            .output()
            .map_err(|e| SystemError::CommandError(format!("Failed to run nmcli: {}", e)))?;

        let stdout = String::from_utf8(output.stdout)?;

        for line in stdout.lines() {
            if line.starts_with("yes:") {
                return Ok(line.trim_start_matches("yes:").to_string());
            }
        }
        Ok("Disconnected".to_string())
    }
}

pub trait ClipboardService {
    fn set_clipboard(&self, text: String) -> std::result::Result<(), SystemError>;
}

#[derive(Clone)]
pub struct LinuxClipboardService;

impl ClipboardService for LinuxClipboardService {
    fn set_clipboard(&self, text: String) -> std::result::Result<(), SystemError> {
        Command::new("wl-copy")
            .arg(text)
            .spawn()
            .map_err(|e| SystemError::CommandError(format!("Failed to run wl-copy: {}", e)))?;
        Ok(())
    }
}

pub trait MediaService {
    fn get_media_info(&self) -> std::result::Result<(String, String), SystemError>;
}

#[derive(Clone)]
pub struct LinuxMediaService;

impl MediaService for LinuxMediaService {
    fn get_media_info(&self) -> std::result::Result<(String, String), SystemError> {
        let get_playerctl_output = |arg: &str| -> std::result::Result<String, SystemError> {
            Command::new("playerctl")
                .arg("metadata")
                .arg(arg)
                .output()
                .map_err(|e| SystemError::CommandError(format!("Failed to run playerctl: {}", e)))
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .map_err(SystemError::Utf8Error)
                            .map(|s| s.trim().to_string())
                    } else {
                        Err(SystemError::CommandError(format!(
                            "playerctl exited with error: {}",
                            String::from_utf8_lossy(&output.stderr)
                        )))
                    }
                })
        };

        let title = get_playerctl_output("title").unwrap_or_default();
        let artist = get_playerctl_output("artist").unwrap_or_default();

        Ok((title, artist))
    }
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let sys = lua.create_table()?;

    let battery_service = LinuxBatteryService;
    sys.set(
        "get_battery",
        lua.create_function(move |lua, ()| {
            let capacity = battery_service.get_capacity()?;
            let status = battery_service.get_status()?;
            let table = lua.create_table_with_capacity(0, 2)?;
            table.set("capacity", capacity)?;
            table.set("status", status)?;
            Ok(Value::Table(table))
        })?,
    )?;

    let wifi_service = LinuxWifiService;
    sys.set(
        "get_wifi_ssid",
        lua.create_async_function(move |lua, ()| {
            let wifi_service_clone = wifi_service.clone();
            async move {
                let ssid_future =
                    tokio::task::spawn_blocking(move || wifi_service_clone.get_ssid());
                let ssid = ssid_future.await.map_err(mlua::Error::external)??;
                Ok(Value::String(lua.create_string(&ssid)?))
            }
        })?,
    )?;

    let clipboard_service = LinuxClipboardService;
    sys.set(
        "set_clipboard",
        lua.create_async_function(move |_, text: String| {
            let clipboard_service_clone = clipboard_service.clone();
            async move {
                tokio::task::spawn_blocking(move || clipboard_service_clone.set_clipboard(text))
                    .await
                    .map_err(|e| {
                        mlua::Error::external(format!("Failed to spawn blocking task: {}", e))
                    })??;
                Ok(Value::Nil)
            }
        })?,
    )?;

    let media_service = LinuxMediaService;
    sys.set(
        "media_info",
        lua.create_async_function(move |lua, ()| {
            let media_service_clone = media_service.clone();
            async move {
                let media_info_future =
                    tokio::task::spawn_blocking(move || media_service_clone.get_media_info());
                let (title, artist) = media_info_future.await.map_err(mlua::Error::external)??;
                let table = lua.create_table_with_capacity(0, 2)?;
                table.set("title", title)?;
                table.set("artist", artist)?;
                Ok(Value::Table(table))
            }
        })?,
    )?;
    lua.globals().set("System", sys)?;
    Ok(())
}
