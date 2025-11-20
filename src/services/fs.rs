use mlua::{Lua, Result};
use std::fs;

pub fn register(lua: &Lua) -> Result<()> {
    let fs_table = lua.create_table()?;

    fs_table.set(
        "read_file",
        lua.create_function(|_, path: String| {
            fs::read_to_string(path).map_err(mlua::Error::external)
        })?,
    )?;

    fs_table.set(
        "write_file",
        lua.create_function(|_, (path, content): (String, String)| {
            fs::write(path, content).map_err(mlua::Error::external)
        })?,
    )?;

    fs_table.set(
        "exists",
        lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).exists()))?,
    )?;

    lua.globals().set("Files", fs_table)?;
    Ok(())
}
