pub mod apps;
pub mod audio;
pub mod system;

use mlua::{Lua, Result};

pub fn init(lua: &Lua) -> Result<()> {
    apps::register(lua)?;
    audio::register(lua)?;
    system::register(lua)?;
    Ok(())
}
