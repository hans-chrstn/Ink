use gtk4::gio::{self, AppInfo, AppLaunchContext};
use gtk4::prelude::*;
use mlua::{Lua, Result, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
struct LuaApp(gio::AppInfo);

impl UserData for LuaApp {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.0.display_name().to_string()));

        fields.add_field_method_get("executable", |_, this| {
            Ok(this.0.executable().to_string_lossy().to_string())
        });

        fields.add_field_method_get("icon", |_, this| {
            Ok(this
                .0
                .icon()
                .and_then(|i| i.to_string())
                .map(|s| s.to_string())
                .unwrap_or_default())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("launch", |_, this, ()| {
            let ctx: Option<&AppLaunchContext> = None;

            if let Err(e) = this.0.launch(&[], ctx) {
                eprintln!("Failed to launch app: {}", e);
            }
            Ok(())
        });
    }
}

pub fn register(lua: &Lua) -> Result<()> {
    let apps = lua.create_table()?;

    apps.set(
        "list",
        lua.create_function(|lua, ()| {
            let all_apps = AppInfo::all();
            let result = lua.create_table()?;
            let mut count = 1;

            for app in all_apps.iter() {
                if !app.should_show() {
                    continue;
                }
                result.set(count, LuaApp(app.clone()))?;
                count += 1;
            }
            Ok(result)
        })?,
    )?;

    lua.globals().set("Apps", apps)?;
    Ok(())
}
