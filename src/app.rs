use crate::core::context::AppContext;
use crate::core::error::{self, AppError};
use crate::scripting::lua_driver::LuaWrapper;
use crate::scripting::traits::ScriptValue;
use crate::ui::builder::UiBuilder;
use crate::ui::strategy::WindowStrategy;

use gtk4::gdk::Display;
use gtk4::{Application, CssProvider, prelude::*};
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

    pub fn setup(&mut self) -> Result<(), AppError> {
        let lua = self.lua.clone();
        let context = self.context.clone();
        let windowed = self.windowed;
        let ui_builder = self.ui_builder.clone();

        let app_clone_for_reload = self.app.clone();
        let lua_clone_for_reload = self.lua.clone();
        let context_clone_for_reload = self.context.clone();
        let ui_builder_clone_for_reload = self.ui_builder.clone();

        self.app.connect_activate(move |app| {
            if let Err(e) =
                Self::load_and_build_ui(app, &lua, &context, windowed, ui_builder.clone())
            {
                eprintln!("Error loading and building UI on activation: {}", e);
            }
        });

        if let Ok(app_global) = self.lua.globals().get::<mlua::Table>("app") {
            app_global.set(
                "reload",
                self.lua.create_function(move |_, ()| {
                    for window in app_clone_for_reload.windows() {
                        window.destroy();
                    }
                    if let Err(e) = Self::load_and_build_ui(
                        &app_clone_for_reload,
                        &lua_clone_for_reload,
                        &context_clone_for_reload,
                        windowed,
                        ui_builder_clone_for_reload.clone(),
                    ) {
                        eprintln!("Error reloading UI: {}", e);
                        return Err(mlua::Error::external(e));
                    }
                    Ok(())
                })?,
            )?;
        }
        Ok(())
    }

    fn load_and_build_ui(
        app: &Application,
        lua: &Rc<Lua>,
        context: &Arc<AppContext>,
        windowed: bool,
        ui_builder: Rc<RefCell<UiBuilder>>,
    ) -> Result<(), AppError> {
        let main_file_path = &context.main_file_path;
        let config_dir = main_file_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        match Self::load_and_execute_lua_config(app, lua, &context.main_file_path) {
            Ok(table) => {
                if let Err(e) = Self::validate_lua_config_table(app, &table) {
                    let app_err = AppError::AppSetupError(e);
                    error::handle_error(app, "Invalid Configuration", &app_err);
                    return Err(app_err);
                }

                Self::setup_lua_actions(app, lua, &table).map_err(AppError::LuaError)?;

                if let Ok(menu_table) = table.get::<mlua::Table>("menu") {
                    let menubar = build_menu_from_lua(menu_table);
                    app.set_menubar(Some(&menubar));
                }

                Self::load_app_css(main_file_path, &table)?;

                Self::build_ui_windows(app, lua, &config_dir, windowed, ui_builder, &table)?;

                if let Ok(on_ready_fn) = lua.globals().get::<mlua::Function>("on_ready")
                    && let Err(e) = on_ready_fn.call::<()>(())
                {
                    let app_err = AppError::LuaError(e);
                    error::handle_error(app, "Lua Error", &app_err);
                    return Err(app_err);
                }
                Ok(())
            }
            Err(e) => {
                let app_err = AppError::LuaError(e);
                error::handle_error(app, "Lua Execution Failed", &app_err);
                Err(app_err)
            }
        }
    }

    fn validate_lua_config_table(_app: &Application, table: &mlua::Table) -> Result<(), String> {
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
        for (key, _) in table.pairs::<String, mlua::Value>().flatten() {
            if !valid_keys.contains(&key.as_str()) && key.parse::<i32>().is_err() {
                return Err(format!("Unknown configuration property: '{}'", key));
            }
        }
        Ok(())
    }

    fn setup_lua_actions(
        app: &Application,
        lua: &Rc<Lua>,
        table: &mlua::Table,
    ) -> mlua::Result<()> {
        if let Ok(actions) = table.get::<mlua::Table>("actions") {
            for action_table in actions.sequence_values::<mlua::Table>().flatten() {
                if let (Ok(name), Ok(callback)) = (
                    action_table.get::<String>("name"),
                    action_table.get::<Function>("callback"),
                ) {
                    let action = gio::SimpleAction::new(&name, None);
                    let lua_rc = lua.clone();
                    let cb_key = lua_rc.create_registry_value(callback)?;
                    action.connect_activate(move |_, _| {
                        if let Ok(cb) = lua_rc.registry_value::<Function>(&cb_key)
                            && let Err(e) = cb.call::<()>(())
                        {
                            eprintln!("Action callback error: {}", e);
                        }
                    });
                    app.add_action(&action);
                }
            }
        }
        Ok(())
    }

    fn load_app_css(main_file_path: &Path, table: &mlua::Table) -> Result<(), AppError> {
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
            let mut css_file = main_file_path
                .parent()
                .ok_or_else(|| {
                    AppError::AppSetupError(
                        "Could not determine parent directory of main file path for CSS."
                            .to_string(),
                    )
                })?
                .to_path_buf();
            css_file.push(rel_path);
            if css_file.exists() {
                let provider = CssProvider::new();
                provider.load_from_path(css_file.to_str().ok_or_else(|| {
                    AppError::AppSetupError(
                        "CSS file path contains invalid Unicode characters.".to_string(),
                    )
                })?);
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
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn build_ui_windows(
        app: &Application,
        lua: &Rc<Lua>,
        config_dir: &Path,
        windowed: bool,
        ui_builder: Rc<RefCell<UiBuilder>>,
        table: &mlua::Table,
    ) -> Result<(), AppError> {
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
            let table = lua.create_table().map_err(AppError::LuaError)?;
            lua.globals()
                .set("app", table.clone())
                .map_err(AppError::LuaError)?;
            table
        };

        let windows_table = lua.create_table().map_err(AppError::LuaError)?;

        for config in window_configs {
            let wrapped = LuaWrapper(config);

            let mut builder = ui_builder.borrow_mut();
            builder
                .register_behavior(
                    "GtkApplicationWindow",
                    Box::new(WindowStrategy::new(windowed)),
                )
                .register_behavior("GtkWindow", Box::new(WindowStrategy::new(windowed)));
            match builder.build(&wrapped, config_dir) {
                Ok(root) => {
                    if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                        w.set_application(Some(app));

                        if let Some(title_prop) = wrapped.get_property("title")
                            && let Some(title_str) = title_prop.as_string()
                        {
                            windows_table
                                .set(
                                    title_str,
                                    crate::scripting::widget_wrapper::LuaWidget(root.clone()),
                                )
                                .map_err(AppError::LuaError)?;
                        }
                    }
                }
                Err(e) => {
                    let app_err = AppError::GtkError(format!("UI Builder Error: {}", e));
                    error::handle_error(app, "UI Builder Error", &app_err);
                    return Err(app_err);
                }
            }
        }

        app_global
            .set("windows", windows_table)
            .map_err(AppError::LuaError)?;

        if let Err(e) = UiBuilder::register_get_widget_by_id_lua_function(lua, &app_global) {
            let app_err = AppError::LuaError(mlua::Error::runtime(format!(
                "Failed to register get_widget_by_id: {}",
                e
            )));
            error::handle_error(app, "Lua Error", &app_err);
            return Err(app_err);
        }
        Ok(())
    }

    fn load_and_execute_lua_config(
        app: &Application,
        lua: &Rc<Lua>,
        main_file_path: &PathBuf,
    ) -> mlua::Result<mlua::Table> {
        let config_dir = main_file_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        if let Some(main_file_str) = main_file_path.to_str() {
            lua.globals()
                .set("INK_MAIN_FILE_PATH", main_file_str)
                .map_err(AppError::LuaError)?;
        }

        let package: mlua::Table = lua.globals().get("package").map_err(AppError::LuaError)?;
        let old_path: String = package.get("path").map_err(AppError::LuaError)?;
        let new_path = format!(
            "{}/?.lua;{}/?/init.lua;{}",
            config_dir.to_string_lossy(),
            config_dir.to_string_lossy(),
            old_path
        );
        package.set("path", new_path).map_err(AppError::LuaError)?;

        if !main_file_path.exists() {
            let app_err = AppError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {:?}", main_file_path),
            ));
            error::handle_error(app, "File Not Found", &app_err);
            return Err(mlua::Error::runtime("File not found"));
        }
        load_lua_script(lua, main_file_path).and_then(|f| {
            f.call::<mlua::Value>(()).and_then(|table_val| {
                if let mlua::Value::Table(table) = table_val {
                    Ok(table)
                } else {
                    Err(mlua::Error::runtime("Lua script must return a UI Table"))
                }
            })
        })
    }
}
fn load_lua_script(lua: &Lua, path: &Path) -> mlua::Result<Function> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_dir = PathBuf::from(home).join(".cache/ink/bytecode");
    let filename = path
        .file_name()
        .ok_or_else(|| mlua::Error::runtime(format!("Invalid path: {:?}", path)))?
        .to_string_lossy();
    let abs_path_hash = match path.canonicalize() {
        Ok(p) => format!("{:x}", md5::compute(p.to_string_lossy().as_bytes())),
        Err(_) => "unknown".to_string(),
    };
    let cache_file = cache_dir.join(format!("{}_{}.luac", filename, abs_path_hash));
    let mut use_cache = false;
    if cache_file.exists()
        && let (Ok(src_meta), Ok(cache_meta)) = (fs::metadata(path), fs::metadata(&cache_file))
        && let (Ok(src_time), Ok(cache_time)) = (src_meta.modified(), cache_meta.modified())
        && cache_time > src_time
    {
        use_cache = true;
    }
    if use_cache {
        let bytes = fs::read(&cache_file).map_err(mlua::Error::external)?;
        lua.load(&bytes).into_function()
    } else {
        let code = fs::read_to_string(path).map_err(mlua::Error::external)?;
        let func = lua.load(&code).set_name(&*filename).into_function()?;
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Failed to create cache directory: {}", e);
        }
        let bytes = func.dump(true);
        let _ = fs::write(&cache_file, bytes);
        Ok(func)
    }
}
fn build_menu_from_lua(menu_table: mlua::Table) -> gio::Menu {
    let menu = gio::Menu::new();
    for item_table in menu_table.sequence_values::<mlua::Table>().flatten() {
        let label = item_table.get::<String>("label").unwrap_or_default();
        if let Ok(submenu_table) = item_table.get::<mlua::Table>("submenu") {
            let submenu = build_menu_from_lua(submenu_table);
            menu.append_submenu(Some(&label), &submenu);
        } else if let Ok(action) = item_table.get::<String>("action") {
            let item = gio::MenuItem::new(Some(&label), Some(&action));
            menu.append_item(&item);
        }
    }
    menu
}
