use crate::core::context::AppContext;
use crate::core::error;
use crate::scripting::lua_driver::LuaWrapper;
use crate::scripting::traits::ScriptValue;
use crate::ui::builder::UiBuilder;
use crate::ui::strategy::WindowStrategy;
use gio;
use gtk4::gdk::Display;
use gtk4::{prelude::*, Application, CssProvider};
use mlua::{Function, Lua};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

pub struct App {
    app: Application,
    lua: Rc<Lua>,
    context: Arc<AppContext>,
    windowed: bool,
    ui_builder: Rc<RefCell<UiBuilder>>,
}

impl App {
    pub fn new(
        app: Application,
        lua: Rc<Lua>,
        context: AppContext,
        windowed: bool,
        ui_builder: Rc<RefCell<UiBuilder>>,
    ) -> Self {
        Self {
            app,
            lua,
            context: Arc::new(context),
            windowed,
            ui_builder,
        }
    }

    pub fn setup(&mut self) {
        let lua = self.lua.clone();
        let context = self.context.clone();
        let windowed = self.windowed;
        let ui_builder = self.ui_builder.clone();

        let app_clone_for_reload = self.app.clone();
        let lua_clone_for_reload = self.lua.clone();
        let context_clone_for_reload = self.context.clone();
        let ui_builder_clone_for_reload = self.ui_builder.clone();

        self.app.connect_activate(move |app| {
            load_and_build_ui(app, &lua, &context, windowed, ui_builder.clone());
        });

        if let Ok(app_global) = self.lua.globals().get::<mlua::Table>("app") {
            app_global
                .set(
                    "reload",
                    self.lua
                        .create_function(move |_, ()| {
                            for window in app_clone_for_reload.windows() {
                                window.destroy();
                            }
                            load_and_build_ui(
                                &app_clone_for_reload,
                                &lua_clone_for_reload,
                                &context_clone_for_reload,
                                windowed,
                                ui_builder_clone_for_reload.clone(),
                            );
                            Ok(())
                        })
                        .unwrap(),
                )
                .unwrap();
        }
    }
}

fn load_and_build_ui(
    app: &Application,
    lua: &Rc<Lua>,
    context: &Arc<AppContext>,
    windowed: bool,
    ui_builder: Rc<RefCell<UiBuilder>>,
) {
    let main_file_path = &context.main_file_path;
    let config_dir = main_file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    if let Some(main_file_str) = main_file_path.to_str() {
        lua.globals()
            .set("INK_MAIN_FILE_PATH", main_file_str)
            .expect("Failed to set INK_MAIN_FILE_PATH");
    }

    let package: mlua::Table = lua
        .globals()
        .get("package")
        .expect("Failed to get package global");
    let old_path: String = package.get("path").expect("Failed to get package.path");
    let new_path = format!(
        "{}/?.lua;{}/?/init.lua;{}",
        config_dir.to_string_lossy(),
        config_dir.to_string_lossy(),
        old_path
    );
    package
        .set("path", new_path)
        .expect("Failed to set package.path");

    if !main_file_path.exists() {
        error::handle_error(
            app,
            "File Not Found",
            &format!("Error: File not found: {:?}", main_file_path),
        );
        return;
    }
    match load_lua_script(&lua, main_file_path).and_then(|f| f.call::<mlua::Value>(())) {
        Ok(table_val) => {
            if let mlua::Value::Table(table) = table_val {
                let valid_keys = [
                    "type",
                    "window_mode",
                    "layer",
                    "anchors",
                    "css_path",
                    "css",
                    "properties",
                    "children",
                    "signals",
                    "keymaps",
                    "margins",
                    "auto_exclusive_zone",
                    "keyboard_mode",
                    "actions",
                    "menu",
                    "id",
                    "realize",
                ];
                for pair in table.pairs::<String, mlua::Value>() {
                    if let Ok((key, _)) = pair {
                        if !valid_keys.contains(&key.as_str()) && key.parse::<i32>().is_err() {
                            let err_msg = format!("Unknown configuration property: '{}'", key);
                            error::handle_error(app, "Invalid Configuration", &err_msg);
                            return;
                        }
                    }
                }

                if let Ok(actions) = table.get::<mlua::Table>("actions") {
                    for action_val in actions.sequence_values::<mlua::Table>() {
                        if let Ok(action_table) = action_val {
                            if let (Ok(name), Ok(callback)) = (
                                action_table.get::<String>("name"),
                                action_table.get::<Function>("callback"),
                            ) {
                                let action = gio::SimpleAction::new(&name, None);
                                let lua_rc = lua.clone();
                                let cb_key = lua_rc.create_registry_value(callback).unwrap();
                                action.connect_activate(move |_, _| {
                                    if let Ok(cb) = lua_rc.registry_value::<Function>(&cb_key) {
                                        if let Err(e) = cb.call::<()>(()) {
                                            eprintln!("Action callback error: {}", e);
                                        }
                                    }
                                });
                                app.add_action(&action);
                            }
                        }
                    }
                }
                if let Ok(menu_table) = table.get::<mlua::Table>("menu") {
                    let menubar = build_menu_from_lua(menu_table);
                    app.set_menubar(Some(&menubar));
                }
                let load_provider = |p: &CssProvider| {
                    if let Some(display) = Display::default() {
                        gtk4::style_context_add_provider_for_display(
                            &display,
                            p,
                            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                        );
                    }
                };
                if let Ok(rel_path) = table.get::<String>("css_path") {
                    let mut css_file = main_file_path.parent().unwrap().to_path_buf();
                    css_file.push(rel_path);
                    if css_file.exists() {
                        let provider = CssProvider::new();
                        provider.load_from_path(css_file.to_str().unwrap());
                        load_provider(&provider);
                    } else {
                        eprintln!("Warn: CSS file not found at {:?}", css_file);
                    }
                }
                if let Ok(css_content) = table.get::<String>("css") {
                    let provider = CssProvider::new();
                    provider.load_from_data(&css_content);
                    load_provider(&provider);
                }
                let first_item = table.raw_get(1);
                let window_configs = if let Ok(mlua::Value::Table(_)) = first_item {
                    table
                        .sequence_values::<mlua::Table>()
                        .filter_map(Result::ok)
                        .map(mlua::Value::Table)
                        .collect()
                } else {
                    vec![mlua::Value::Table(table.clone())]
                };

                let app_global: mlua::Table = if let Ok(table) = lua.globals().get("app") {
                    table
                } else {
                    let table = lua
                        .create_table()
                        .expect("Failed to create app global table");
                    lua.globals()
                        .set("app", table.clone())
                        .expect("Failed to set app global table");
                    table
                };

                let windows_table = lua
                    .create_table()
                    .expect("Failed to create Lua table for windows");

                for config in window_configs {
                    let wrapped = LuaWrapper(config);

                    let mut builder = ui_builder.borrow_mut();
                    builder
                        .register_behavior(
                            "GtkApplicationWindow",
                            Box::new(WindowStrategy::new(windowed)),
                        )
                        .register_behavior("GtkWindow", Box::new(WindowStrategy::new(windowed)));
                    match builder.build(&wrapped, &config_dir) {
                        Ok(root) => {
                            if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                                w.set_application(Some(app));

                                if let Some(title_prop) = wrapped.get_property("title") {
                                    if let Some(title_str) = title_prop.as_string() {
                                        windows_table
                                            .set(
                                                title_str,
                                                crate::scripting::widget_wrapper::LuaWidget(
                                                    root.clone(),
                                                ),
                                            )
                                            .expect("Failed to store window in Lua table");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error::handle_error(app, "UI Builder Error", e.as_str());
                        }
                    }
                }

                app_global
                    .set("windows", windows_table)
                    .expect("Failed to set app.windows global");

                if let Err(e) = UiBuilder::register_get_widget_by_id_lua_function(lua, &app_global)
                {
                    error::handle_error(
                        app,
                        "Lua Error",
                        &format!("Failed to register get_widget_by_id: {}", e),
                    );
                    return;
                }

                if let Ok(on_ready_fn) = app_global.get::<mlua::Function>("on_ready") {
                    if let Err(e) = on_ready_fn.call::<()>(()) {
                        error::handle_error(
                            app,
                            "Lua Error",
                            &format!("Error calling app.on_ready: {}", e),
                        );
                        return;
                    }
                }
            } else {
                error::handle_error(
                    app,
                    "Invalid Configuration",
                    "Lua script must return a UI Table",
                );
            }
        }
        Err(e) => {
            error::handle_error(app, "Lua Execution Failed", &e.to_string());
        }
    }
}
fn load_lua_script(lua: &Lua, path: &Path) -> mlua::Result<Function> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_dir = PathBuf::from(home).join(".cache/ink/bytecode");
    let filename = path.file_name().unwrap().to_string_lossy();
    let abs_path_hash = match path.canonicalize() {
        Ok(p) => format!("{:x}", md5::compute(p.to_string_lossy().as_bytes())),
        Err(_) => "unknown".to_string(),
    };
    let cache_file = cache_dir.join(format!("{}_{}.luac", filename, abs_path_hash));
    let mut use_cache = false;
    if cache_file.exists() {
        if let (Ok(src_meta), Ok(cache_meta)) = (fs::metadata(path), fs::metadata(&cache_file)) {
            if let (Ok(src_time), Ok(cache_time)) = (src_meta.modified(), cache_meta.modified()) {
                if cache_time > src_time {
                    use_cache = true;
                }
            }
        }
    }
    if use_cache {
        let bytes = fs::read(&cache_file).map_err(mlua::Error::external)?;
        lua.load(&bytes).into_function()
    } else {
        let code = fs::read_to_string(path).map_err(mlua::Error::external)?;
        let func = lua.load(&code).set_name(filename).into_function()?;
        if let Err(_) = fs::create_dir_all(&cache_dir) {
            return Ok(func);
        }
        let bytes = func.dump(true);
        let _ = fs::write(&cache_file, bytes);
        Ok(func)
    }
}
fn build_menu_from_lua(menu_table: mlua::Table) -> gio::Menu {
    let menu = gio::Menu::new();
    for item_val in menu_table.sequence_values::<mlua::Table>() {
        if let Ok(item_table) = item_val {
            let label = item_table.get::<String>("label").unwrap_or_default();
            if let Ok(submenu_table) = item_table.get::<mlua::Table>("submenu") {
                let submenu = build_menu_from_lua(submenu_table);
                menu.append_submenu(Some(&label), &submenu);
            } else if let Ok(action) = item_table.get::<String>("action") {
                let item = gio::MenuItem::new(Some(&label), Some(&action));
                menu.append_item(&item);
            }
        }
    }
    menu
}
