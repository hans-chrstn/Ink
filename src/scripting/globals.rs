use crate::scripting::lua_driver::LuaWrapper;
use crate::scripting::stdlib;
use crate::scripting::widget_wrapper::{LuaGType, LuaWidget};
use crate::services;
use crate::ui::builder::UiBuilder;
use gtk4::Application;
use gtk4::gdk;
use gtk4::glib;
use gtk4::glib::prelude::*;
use gtk4::prelude::*;
use mlua::{Function, Lua, Result, Value};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

fn init_gtk_bindings(lua: &Rc<Lua>, globals: &mlua::Table) -> Result<()> {
    let gtk_table = lua.create_table()?;
    gtk_table.set("Window", LuaGType(gtk4::Window::static_type()))?;
    globals.set("Gtk", gtk_table)?;
    Ok(())
}

fn init_ink_core_functions(
    lua: &Rc<Lua>,
    ink_table: &mlua::Table,
    ui_builder: Rc<RefCell<UiBuilder>>,
) -> Result<()> {
    ink_table.set(
        "get_widget_by_id",
        lua.create_function({
            let ui_builder = ui_builder.clone();
            move |lua, id: String| {
                if let Some(widget) = ui_builder.borrow().get_widget_by_id(&id) {
                    Ok(mlua::Value::UserData(
                        lua.create_userdata(LuaWidget(widget))?,
                    ))
                } else {
                    Ok(mlua::Value::Nil)
                }
            }
        })?,
    )?;

    let tray_table = lua.create_table()?;
    ink_table.set("tray", tray_table)?;

    ink_table.set(
        "markdown_to_pango",
        lua.create_function(|_, markdown: String| Ok(stdlib::markdown_to_pango(&markdown)))?,
    )?;
    Ok(())
}

fn init_clipboard_functions(lua: &Rc<Lua>, globals: &mlua::Table) -> Result<()> {
    let clipboard_table = lua.create_table()?;
    clipboard_table.set(
        "set_text",
        lua.create_function(|_, text: String| {
            let display = gdk::Display::default().expect("Could not get default display");
            display.clipboard().set_text(&text);
            Ok(())
        })?,
    )?;

    clipboard_table.set(
        "read_text",
        lua.create_function({
            let lua = lua.clone();
            move |_, callback: Function| {
                let display = gdk::Display::default().expect("Could not get default display");
                let clipboard = display.clipboard();
                let cb_key = lua.create_registry_value(callback)?;
                let lua_clone = lua.clone();

                clipboard.read_text_async(None::<&gio::Cancellable>, move |res| {
                    if let Ok(func) = lua_clone.registry_value::<Function>(&cb_key) {
                        match res {
                            Ok(Some(text)) => {
                                let _ = func.call::<()>(text.to_string());
                            }
                            _ => {
                                let _ = func.call::<()>(mlua::Nil);
                            }
                        }
                    }
                });
                Ok(())
            }
        })?,
    )?;
    globals.set("Clipboard", clipboard_table)?;
    Ok(())
}

fn init_ui_builder_function(
    lua: &Rc<Lua>,
    globals: &mlua::Table,
    app: Application,
    config_dir: PathBuf,
    ui_builder: Rc<RefCell<UiBuilder>>,
) -> Result<()> {
    let build_ui = lua.create_function({
        let app = app.clone();
        let config_dir = config_dir.clone();
        let ui_builder = ui_builder.clone();
        move |_, config: Value| {
            let wrapped = LuaWrapper(config);

            match ui_builder.borrow().build(&wrapped, &config_dir) {
                Ok(root) => {
                    if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                        w.set_application(Some(&app));
                        w.show();
                    }
                    Ok(LuaWidget(root))
                }
                Err(e) => Err(mlua::Error::RuntimeError(e)),
            }
        }
    })?;
    globals.set("build_ui", build_ui)?;
    Ok(())
}

fn init_notification_function(lua: &Rc<Lua>, globals: &mlua::Table) -> Result<()> {
    let notify = lua.create_function(|lua, (summary, body): (String, Option<String>)| {
        let build_ui = lua.globals().get::<Function>("build_ui")?;
        let notif_table = lua.create_table()?;
        notif_table.set("type", "GtkApplicationWindow")?;
        notif_table.set("window_mode", "layer_shell")?;
        notif_table.set("layer", "top")?;
        notif_table.set("auto_exclusive_zone", true)?;
        let anchors = lua.create_table()?;
        anchors.set("top", true)?;
        anchors.set("right", true)?;
        notif_table.set("anchors", anchors)?;
        let margins = lua.create_table()?;
        margins.set("top", 10)?;
        margins.set("right", 10)?;
        notif_table.set("margins", margins)?;
        notif_table.set(
            "css",
            r#"
            window {
                background-color: rgba(30, 30, 40, 0.9);
                border-radius: 12px;
                border: 1px solid rgba(120, 120, 150, 0.8);
            }
            label {
                color: white;
            }
            .summary {
                font-size: 1.1em;
                font-weight: bold;
            }
            .body {
                font-size: 1.0em;
            }
        "#,
        )?;
        let children_table = lua.create_table()?;
        let container = lua.create_table()?;
        container.set("type", "GtkBox")?;
        let container_props = lua.create_table()?;
        container_props.set("orientation", "vertical")?;
        container_props.set("spacing", 6)?;
        container_props.set("margin_top", 12)?;
        container_props.set("margin_bottom", 12)?;
        container_props.set("margin_start", 16)?;
        container_props.set("margin_end", 16)?;
        container.set("properties", container_props)?;

        let container_children = lua.create_table()?;
        let summary_label = lua.create_table()?;
        summary_label.set("type", "GtkLabel")?;
        let summary_props = lua.create_table()?;
        summary_props.set("label", summary)?;
        summary_props.set("halign", "start")?;
        summary_props.set("css_classes", lua.create_sequence_from(vec!["summary"])?)?;
        summary_label.set("properties", summary_props)?;
        container_children.raw_insert(1, summary_label)?;
        if let Some(body_text) = body {
            let body_label = lua.create_table()?;
            body_label.set("type", "GtkLabel")?;
            let body_props = lua.create_table()?;
            body_props.set("label", body_text)?;
            body_props.set("halign", "start")?;
            body_props.set("wrap", true)?;
            body_props.set("css_classes", lua.create_sequence_from(vec!["body"])?)?;
            body_label.set("properties", body_props)?;
            container_children.raw_insert(2, body_label)?;
        }

        container.set("children", container_children)?;
        children_table.raw_insert(1, container)?;
        notif_table.set("children", children_table)?;
        let window_widget: LuaWidget = build_ui.call(notif_table)?;
        glib::timeout_add_local_once(std::time::Duration::from_secs(5), move || {
            if let Ok(window) = window_widget.0.downcast::<gtk4::Window>() {
                window.destroy();
            }
        });
        Ok(())
    })?;
    globals.set("notify", notify)?;
    Ok(())
}

fn init_utility_functions(lua: &Rc<Lua>, globals: &mlua::Table) -> Result<()> {
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
    let fetch = lua.create_function(
        |_, (method, uri, headers, body): (String, String, Option<mlua::Table>, Option<String>)| {
            let headers_map: Option<std::collections::HashMap<String, String>> =
                headers.map(|t| t.pairs::<String, String>().filter_map(Result::ok).collect());
            stdlib::fetch(&method, &uri, headers_map, body).map_err(mlua::Error::RuntimeError)
        },
    )?;
    globals.set("fetch", fetch)?;

    let fetch_async = lua.create_function(
        |lua,
         (method, uri, headers, body, callback): (
            String,
            String,
            Option<mlua::Table>,
            Option<String>,
            Function,
        )| {
            let headers_map: Option<std::collections::HashMap<String, String>> =
                headers.map(|t| t.pairs::<String, String>().filter_map(Result::ok).collect());

            let lua = lua.clone();
            let cb_key = lua.create_registry_value(callback)?;

            glib::MainContext::default().spawn_local(async move {
                let result = stdlib::fetch_async(method, uri, headers_map, body).await;
                if let Ok(func) = lua.registry_value::<Function>(&cb_key) {
                    let lua_result_table = lua
                        .create_table()
                        .expect("Failed to create Lua table for result");

                    match result {
                        Ok(s) => {
                            lua_result_table
                                .set("ok", s)
                                .expect("Failed to set 'ok' field");
                        }
                        Err(e) => {
                            lua_result_table
                                .set("err", e)
                                .expect("Failed to set 'err' field");
                        }
                    }
                    func.call::<()>(lua_result_table)
                        .expect("Error in Lua fetch_async callback");
                }
            });

            Ok(())
        },
    )?;
    globals.set("fetch_async", fetch_async)?;
    let spawn = lua.create_function(|_, cmd: String| {
        stdlib::spawn(cmd);
        Ok(())
    })?;
    globals.set("spawn", spawn)?;
    Ok(())
}

pub fn init(
    lua: Rc<Lua>,
    app: Application,
    config_dir: PathBuf,
    ui_builder: Rc<RefCell<UiBuilder>>,
) -> Result<()> {
    let globals = lua.globals();

    let ink_table = lua.create_table()?;
    globals.set("ink", ink_table.clone())?;

    init_gtk_bindings(&lua, &globals)?;
    init_ink_core_functions(&lua, &ink_table, ui_builder.clone())?;
    init_clipboard_functions(&lua, &globals)?;
    init_ui_builder_function(
        &lua,
        &globals,
        app.clone(),
        config_dir.clone(),
        ui_builder.clone(),
    )?;
    init_notification_function(&lua, &globals)?;
    init_utility_functions(&lua, &globals)?;

    services::init(lua.clone())?;
    Ok(())
}
