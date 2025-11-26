use image::{ImageBuffer, ImageFormat, Rgba};
use mlua::{Lua, Result, Table};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;
use zbus::blocking::Proxy;
use zbus::zvariant::{Type, Value};

use super::desktop_entry;

#[derive(Debug)]
pub enum TrayApiError {
    DbusError(zbus::Error),
    IoError(std::io::Error),
    ImageError(image::ImageError),
    PixmapConversionError(String),
    DesktopEntryError(String),
    Other(String),
}

impl std::fmt::Display for TrayApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayApiError::DbusError(e) => write!(f, "DBus error: {}", e),
            TrayApiError::IoError(e) => write!(f, "I/O error: {}", e),
            TrayApiError::ImageError(e) => write!(f, "Image error: {}", e),
            TrayApiError::PixmapConversionError(e) => write!(f, "Pixmap conversion error: {}", e),
            TrayApiError::DesktopEntryError(e) => write!(f, "Desktop entry error: {}", e),
            TrayApiError::Other(e) => write!(f, "Tray API error: {}", e),
        }
    }
}

impl StdError for TrayApiError {}

impl From<zbus::Error> for TrayApiError {
    fn from(err: zbus::Error) -> Self {
        TrayApiError::DbusError(err)
    }
}

impl From<std::io::Error> for TrayApiError {
    fn from(err: std::io::Error) -> Self {
        TrayApiError::IoError(err)
    }
}

impl From<image::ImageError> for TrayApiError {
    fn from(err: image::ImageError) -> Self {
        TrayApiError::ImageError(err)
    }
}

impl From<TrayApiError> for mlua::Error {
    fn from(err: TrayApiError) -> Self {
        mlua::Error::external(err)
    }
}

#[derive(Debug, Deserialize, Type)]
pub struct Pixmap {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct StatusNotifierItemClient<'a> {
    proxy: Proxy<'a>,
}

impl<'a> StatusNotifierItemClient<'a> {
    pub fn new(service: &'a str) -> std::result::Result<Self, TrayApiError> {
        let conn = zbus::blocking::Connection::session()?;
        let proxy = Proxy::new(
            &conn,
            service,
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
        )?;
        Ok(Self { proxy })
    }

    pub fn get_property_string(
        &self,
        property_name: &str,
    ) -> std::result::Result<String, TrayApiError> {
        self.proxy
            .get_property(property_name)
            .map_err(TrayApiError::DbusError)
    }

    pub fn get_property_value(
        &self,
        property_name: &str,
    ) -> std::result::Result<Value<'_>, TrayApiError> {
        self.proxy
            .get_property(property_name)
            .map_err(TrayApiError::DbusError)
    }
}

fn get_item_properties_raw_impl(
    service: &str,
) -> std::result::Result<HashMap<String, String>, TrayApiError> {
    let client = StatusNotifierItemClient::new(service)?;

    let id: String = client.get_property_string("Id")?;
    let title: String = client.get_property_string("Title")?;
    let icon_name: String = client.get_property_string("IconName")?;
    let status: String = client.get_property_string("Status")?;
    let tool_tip: String = client.get_property_string("ToolTip")?;
    let desktop_item_path: String =
        client.get_property_string("X-KDE-StatusNotifierItem-DesktopItem")?;

    let mut props = HashMap::new();
    props.insert("service".to_string(), service.to_string());
    props.insert("id".to_string(), id);
    props.insert("title".to_string(), title);
    props.insert("icon_name".to_string(), icon_name);
    props.insert("status".to_string(), status);
    props.insert("tool_tip".to_string(), tool_tip);
    props.insert("desktop_item".to_string(), desktop_item_path);

    Ok(props)
}

fn parse_icon_pixmap(pixmap_value: Value) -> std::result::Result<Option<Pixmap>, TrayApiError> {
    let pixmaps: Vec<Pixmap> = if let zbus::zvariant::Value::Array(array) = pixmap_value {
        array
            .iter()
            .filter_map(|value| {
                if let zbus::zvariant::Value::Structure(s) = value {
                    let fields = s.fields();
                    if fields.len() == 3 {
                        let width = if let zbus::zvariant::Value::I32(v) = fields[0] {
                            v
                        } else {
                            return None;
                        };
                        let height = if let zbus::zvariant::Value::I32(v) = fields[1] {
                            v
                        } else {
                            return None;
                        };
                        let data_array = if let zbus::zvariant::Value::Array(v) = &fields[2] {
                            v
                        } else {
                            return None;
                        };

                        let data: Vec<u8> = data_array
                            .iter()
                            .filter_map(|v| {
                                if let zbus::zvariant::Value::U8(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        Some(Pixmap {
                            width,
                            height,
                            data,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(pixmaps.into_iter().next())
}

fn save_pixmap_to_temp_file(
    pixmap: &Pixmap,
    service: &str,
) -> std::result::Result<PathBuf, TrayApiError> {
    let mut final_path = PathBuf::new();
    let file_name = format!("{}.png", service);
    final_path.push(std::env::temp_dir());
    final_path.push("ink");

    if !final_path.exists() {
        std::fs::create_dir_all(&final_path)?;
    }

    final_path.push(&file_name);

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
        pixmap.width as u32,
        pixmap.height as u32,
        pixmap.data.clone(),
    )
    .ok_or_else(|| {
        TrayApiError::PixmapConversionError("Failed to create image buffer".to_string())
    })?;

    let mut output = File::create(&final_path)?;
    image.write_to(&mut output, ImageFormat::Png)?;
    Ok(final_path)
}

pub(crate) fn get_item_properties_processed(
    service: &str,
) -> std::result::Result<HashMap<String, String>, TrayApiError> {
    let client = StatusNotifierItemClient::new(service)?;

    let id: String = client.get_property_string("Id")?;
    let mut title: String = client.get_property_string("Title")?;
    let mut icon_name: String = client.get_property_string("IconName")?;
    let status: String = client.get_property_string("Status")?;
    let tool_tip: String = client.get_property_string("ToolTip")?;
    let desktop_item_path: String =
        client.get_property_string("X-KDE-StatusNotifierItem-DesktopItem")?;

    if title.is_empty() {
        title = id.clone();
    }

    let mut props = HashMap::new();
    props.insert("service".to_string(), service.to_string());
    props.insert("id".to_string(), id);
    props.insert("status".to_string(), status);
    props.insert("tool_tip".to_string(), tool_tip);

    if !desktop_item_path.is_empty() {
        props.insert("desktop_item".to_string(), desktop_item_path.clone());
        if let Some(entry) = desktop_entry::DesktopEntry::parse_from_file(&desktop_item_path) {
            if let Some(app_name) = entry.name {
                title = app_name;
            }
            if let Some(app_icon) = entry.icon {
                icon_name = app_icon;
            }
        } else {
            return Err(TrayApiError::DesktopEntryError(format!(
                "Failed to parse desktop entry: {}",
                desktop_item_path
            )));
        }
    }
    props.insert("title".to_string(), title);

    if icon_name.is_empty() {
        let pixmap_value = client.get_property_value("IconPixmap")?;
        if let Some(pixmap) = parse_icon_pixmap(pixmap_value)? {
            let saved_path = save_pixmap_to_temp_file(&pixmap, service)?;
            props.insert(
                "icon_path".to_string(),
                saved_path
                    .to_str()
                    .ok_or_else(|| {
                        TrayApiError::Other("Failed to convert path to string".to_string())
                    })?
                    .to_string(),
            );
        }
    } else {
        props.insert("icon_name".to_string(), icon_name);
    }

    Ok(props)
}

pub fn register(lua: Rc<Lua>) -> Result<()> {
    let globals = lua.globals();
    let app_table: Table = globals.get("app")?;
    let tray_table: Table = app_table.get("tray")?;

    tray_table.set(
        "get_item_properties",
        lua.create_function(|lua, service: String| {
            match get_item_properties_processed(&service) {
                Ok(props) => {
                    let table = lua.create_table()?;
                    for (k, v) in props {
                        table.set(k, v)?;
                    }
                    Ok(table)
                }
                Err(e) => Err(mlua::Error::external(e)),
            }
        })?,
    )?;

    tray_table.set(
        "get_item_raw_properties",
        lua.create_function(
            |lua, service: String| match get_item_properties_raw_impl(&service) {
                Ok(props) => {
                    let table = lua.create_table()?;
                    for (k, v) in props {
                        table.set(k, v)?;
                    }
                    Ok(table)
                }
                Err(e) => Err(mlua::Error::external(e)),
            },
        )?,
    )?;

    Ok(())
}
