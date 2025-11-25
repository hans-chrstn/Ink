use image::{ImageBuffer, ImageFormat, Rgba};
use mlua::{Lua, Result, Table};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;
use zbus::blocking::Proxy;
use zbus::zvariant::{Type, Value};

use super::desktop_entry;

#[derive(Debug, Deserialize, Type)]
pub struct Pixmap {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

fn get_item_properties_raw_impl(
    service: &str,
) -> std::result::Result<HashMap<String, String>, Box<dyn Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let proxy = Proxy::new(
        &conn,
        service,
        "/StatusNotifierItem",
        "org.kde.StatusNotifierItem",
    )?;

    let id: String = proxy.get_property("Id").unwrap_or_default();
    let title: String = proxy.get_property("Title").unwrap_or_default();
    let icon_name: String = proxy.get_property("IconName").unwrap_or_default();
    let status: String = proxy.get_property("Status").unwrap_or_default();
    let tool_tip: String = proxy.get_property("ToolTip").unwrap_or_default();
    let desktop_item_path: String = proxy
        .get_property("X-KDE-StatusNotifierItem-DesktopItem")
        .unwrap_or_default();

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

pub(crate) fn get_item_properties_processed(
    service: &str,
) -> std::result::Result<HashMap<String, String>, Box<dyn Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let proxy = Proxy::new(
        &conn,
        service,
        "/StatusNotifierItem",
        "org.kde.StatusNotifierItem",
    )?;

    let id: String = proxy.get_property("Id").unwrap_or_default();
    let mut title: String = proxy.get_property("Title").unwrap_or_default();
    let mut icon_name: String = proxy.get_property("IconName").unwrap_or_default();
    let status: String = proxy.get_property("Status").unwrap_or_default();
    let tool_tip: String = proxy.get_property("ToolTip").unwrap_or_default();
    let desktop_item_path: String = proxy
        .get_property("X-KDE-StatusNotifierItem-DesktopItem")
        .unwrap_or_default();

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
        }
    }
    props.insert("title".to_string(), title);

    if icon_name.is_empty() {
        if let Ok(pixmap_value) = proxy.get_property::<Value>("IconPixmap") {
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
                                let data_array = if let zbus::zvariant::Value::Array(v) = &fields[2]
                                {
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

            if let Some(pixmap) = pixmaps.get(0) {
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
                .ok_or("Failed to create image buffer")?;

                let mut output = File::create(&final_path)?;
                image.write_to(&mut output, ImageFormat::Png)?;
                props.insert(
                    "icon_path".to_string(),
                    final_path.to_str().unwrap().to_string(),
                );
            }
        }
    } else {
        props.insert("icon_name".to_string(), icon_name);
    }

    Ok(props)
}

pub fn register(lua: Rc<Lua>) -> Result<()> {
    let globals = lua.globals();
    let ink_table: Table = globals.get("ink")?;
    let tray_table: Table = ink_table.get("tray")?;

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
