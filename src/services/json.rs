use mlua::{Lua, LuaSerdeExt, Result, Table};
use serde_json::Value as JsonValue;
pub fn register(lua: &Lua) -> Result<()> {
    let json_table = lua.create_table()?;
    json_table.set(
        "stringify",
        lua.create_function(|lua, value: mlua::Value| -> Result<mlua::Value> {
            serde_json::to_string(&value)
                .map_err(mlua::Error::external)
                .and_then(|s| Ok(mlua::Value::String(lua.create_string(&s)?)))
        })?,
    )?;
    json_table.set(
        "parse",
        lua.create_function(|lua, str: String| {
            let v: JsonValue = serde_json::from_str(&str).map_err(mlua::Error::external)?;
            Ok(lua.to_value(&v))
        })?,
    )?;
    lua.globals().get::<Table>("ink")?.set("json", json_table)?;
    Ok(())
}
