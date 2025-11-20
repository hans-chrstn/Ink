pub mod apps;
pub mod audio;
pub mod fs;
pub mod json;
pub mod system;

use mlua::{Lua, Result};
use std::rc::Rc;

pub fn init(lua: Rc<Lua>) -> Result<()> {
    apps::register(&lua)?;
    audio::register(lua.clone())?;
    system::register(&lua)?;
    json::register(&lua)?;
    fs::register(&lua)?;
    Ok(())
}
