use crate::scripting::stdlib;
use mlua::{Lua, Result};

pub fn init(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let exec = lua
        .create_function(|_, cmd: String| stdlib::exec(&cmd).map_err(mlua::Error::RuntimeError))?;
    globals.set("exec", exec)?;

    let spawn = lua.create_function(|_, cmd: String| {
        stdlib::spawn(cmd);
        Ok(())
    })?;
    globals.set("spawn", spawn)?;

    let fetch = lua
        .create_function(|_, url: String| stdlib::fetch(&url).map_err(mlua::Error::RuntimeError))?;
    globals.set("fetch", fetch)?;

    Ok(())
}
