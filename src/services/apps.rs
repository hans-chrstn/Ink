use gtk4::gio::AppInfo;
use gtk4::prelude::*;
use mlua::{Lua, Result};

pub fn register(lua: &Lua) -> Result<()> {
    let apps = lua.create_table()?;

    apps.set(
        "list",
        lua.create_function(|lua, ()| {
            let all_apps = AppInfo::all();
            let result = lua.create_table()?;

            for (i, app) in all_apps.iter().enumerate() {
                if !app.should_show() {
                    continue;
                }

                let entry = lua.create_table()?;
                entry.set("name", app.display_name().to_string())?;

                entry.set("executable", app.executable().to_string_lossy().to_string())?;

                let icon_str = app
                    .icon()
                    .and_then(|i| i.to_string())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                entry.set("icon", icon_str)?;

                result.set(i + 1, entry)?;
            }
            Ok(result)
        })?,
    )?;

    apps.set(
        "launch",
        lua.create_function(|_, command: String| {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(())
        })?,
    )?;

    lua.globals().set("Apps", apps)?;
    Ok(())
}
