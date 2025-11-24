use mlua::{Lua, LuaSerdeExt, Result};
use serde_json::Value as JsonValue;
pub fn register(lua: &Lua) -> Result<()> {
    let json = lua.create_table()?;
    json.set(
        "parse",
        lua.create_function(|lua, str: String| {
            let v: JsonValue = serde_json::from_str(&str).map_err(mlua::Error::external)?;
            Ok(lua.to_value(&v))
        })?,
    )?;
    lua.globals().set("JSON", json)?;
    Ok(())
}
