pub mod apps;
pub mod audio;
pub mod dbus_service;
pub mod desktop_entry;
pub mod fs;
pub mod json;
pub mod system;
pub mod tray_api;

use mlua::{Lua, Result};
use std::rc::Rc;

pub fn init(lua: Rc<Lua>) -> Result<()> {
    apps::register(&lua)?;
    audio::register(lua.clone())?;
    system::register(&lua)?;
    json::register(&lua)?;
    fs::register(lua.clone())?;
    dbus_service::init(lua.clone()).map_err(mlua::Error::external)?;
    tray_api::register(lua.clone())?;
    Ok(())
}
