use crate::scripting::lua_driver::LuaWrapper;
use crate::scripting::stdlib;
use crate::scripting::widget_wrapper::LuaWidget;
use crate::services;
use crate::ui::builder::UiBuilder;
use gtk4::glib;
use mlua::{Function, Lua, Result, Value};
use std::rc::Rc;

use gtk4::prelude::*;
use crate::ui::strategy::WindowStrategy;
use gtk4::Application;

pub fn init(lua: Rc<Lua>, app: Application) -> Result<()> {
    let globals = lua.globals();

    let build_ui = lua.create_function(move |_, config: Value| {
        let wrapped = LuaWrapper(config);
        let builder = UiBuilder::new()
            .register_behavior(
                "GtkApplicationWindow",
                Box::new(WindowStrategy::new(false)),
            )
            .register_behavior("GtkWindow", Box::new(WindowStrategy::new(false)));

        match builder.build(&wrapped) {
            Ok(root) => {
                if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                    w.set_application(Some(&app));
                    w.show();
                }
                Ok(LuaWidget(root))
            }
            Err(e) => Err(mlua::Error::RuntimeError(e)),
        }
    })?;
    globals.set("build_ui", build_ui)?;

    let notify = lua.create_function(|_, (summary, body): (String, Option<String>)| {
        let mut notification = notify_rust::Notification::new();
        notification.summary(&summary);
        if let Some(body) = body {
            notification.body(&body);
        }
        notification.show().map_err(mlua::Error::external)?;
        Ok(())
    })?;
    globals.set("notify", notify)?;


    let exit = lua.create_function(|_, code: Option<i32>| -> Result<()> {
        std::process::exit(code.unwrap_or(0));
    })?;
    globals.set("exit", exit)?;

    let set_interval = lua.create_function(|lua, (ms, callback): (u32, Function)| {
        let lua = lua.clone();
        let cb_key = lua.create_registry_value(callback)?;

        glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
            if let Ok(func) = lua.registry_value::<Function>(&cb_key) {
                if let Err(e) = func.call::<()>(()) {
                    eprintln!("Interval Error: {}", e);
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
        Ok(())
    })?;
    globals.set("set_interval", set_interval)?;

    let set_timeout = lua.create_function(|lua, (ms, callback): (u32, Function)| {
        let lua = lua.clone();
        let cb_key = lua.create_registry_value(callback)?;

        glib::timeout_add_local_once(std::time::Duration::from_millis(ms as u64), move || {
            if let Ok(func) = lua.registry_value::<Function>(&cb_key) {
                if let Err(e) = func.call::<()>(()) {
                    eprintln!("setTimeout Error: {}", e);
                }
            }
        });
        Ok(())
    })?;
    globals.set("set_timeout", set_timeout)?;

    let exec = lua
        .create_function(|_, cmd: String| stdlib::exec(&cmd).map_err(mlua::Error::RuntimeError))?;
    globals.set("exec", exec)?;

    {
        let lua = lua.clone();
        let exec_async = lua.create_function(move |lua, (cmd, callback): (String, Function)| {
            let lua = lua.clone();
            let cb_key = lua.create_registry_value(callback)?;
            glib::MainContext::default().spawn_local(async move {
                let result = stdlib::exec_async(cmd).await;
                if let Ok(func) = lua.registry_value::<Function>(&cb_key) {
                    let _ = func.call::<()>(result);
                }
            });
            Ok(())
        })?;
        globals.set("exec_async", exec_async)?;
    }

    let fetch = lua
        .create_function(|_, url: String| stdlib::fetch(&url).map_err(mlua::Error::RuntimeError))?;
    globals.set("fetch", fetch)?;

    {
        let lua = lua.clone();
        let fetch_async =
            lua.create_function(move |lua, (url, callback): (String, Function)| {
                let lua = lua.clone();
                let cb_key = lua.create_registry_value(callback)?;
                glib::MainContext::default().spawn_local(async move {
                    let result = stdlib::fetch_async(url).await;
                    if let Ok(func) = lua.registry_value::<Function>(&cb_key) {
                        let _ = func.call::<()>(result);
                    }
                });
                Ok(())
            })?;
        globals.set("fetch_async", fetch_async)?;
    }

    let spawn = lua.create_function(|_, cmd: String| {
        stdlib::spawn(cmd);
        Ok(())
    })?;
    globals.set("spawn", spawn)?;

    services::init(lua.clone())?;

    Ok(())
}
