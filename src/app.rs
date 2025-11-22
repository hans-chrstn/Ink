use crate::core::context::AppContext;
use crate::scripting::globals;
use crate::scripting::lua_driver::LuaWrapper;
use crate::ui::builder::UiBuilder;
use crate::ui::strategy::WindowStrategy;
use gtk4::gdk::Display;
use gtk4::{Application, CssProvider, prelude::*};
use mlua::{Function, Lua, Table};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

pub struct App {
    app: Application,
    lua: Rc<Lua>,
    context: Arc<AppContext>,
    windowed: bool,
}

impl App {
    pub fn new(app: Application, context: AppContext, windowed: bool) -> Self {
        let lua = Rc::new(Lua::new());

        globals::init(lua.clone(), app.clone()).expect("Failed to initialize Lua globals");

        Self {
            app,
            lua,
            context: Arc::new(context),
            windowed,
        }
    }

    pub fn setup(&mut self) {
        let lua = self.lua.clone();
        let context = self.context.clone();
        let windowed = self.windowed;

        self.app.connect_activate(move |app| {
            let main_file_path = &context.main_file_path;

            if let Some(parent) = main_file_path.parent().and_then(|p| p.to_str()) {
                let _ = lua.globals().get::<Table>("package").unwrap().set(
                    "path",
                    format!("{};{}/?.lua;{}/?/init.lua", "", parent, parent),
                );
            }

            if let Some(main_file_str) = main_file_path.to_str() {
                lua.globals()
                    .set("INK_MAIN_FILE_PATH", main_file_str)
                    .expect("Failed to set INK_MAIN_FILE_PATH");
            }

            if !main_file_path.exists() {
                eprintln!("Error: File not found: {:?}", main_file_path);
                return;
            }

            match load_lua_script(&lua, main_file_path).and_then(|f| f.call::<mlua::Value>(())) {
                Ok(table_val) => {
                    if let mlua::Value::Table(table) = table_val {
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

                        for config in window_configs {
                            let wrapped = LuaWrapper(config);
                            let builder = UiBuilder::new()
                                .register_behavior(
                                    "GtkApplicationWindow",
                                    Box::new(WindowStrategy::new(windowed)),
                                )
                                .register_behavior(
                                    "GtkWindow",
                                    Box::new(WindowStrategy::new(windowed)),
                                );

                            match builder.build(&wrapped) {
                                Ok(root) => {
                                    if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                                        w.set_application(Some(app));
                                        w.show();
                                    }
                                }
                                Err(e) => eprintln!("Builder Error: {}", e),
                            }
                        }
                    } else {
                        eprintln!("Error: Lua script must return a UI Table");
                    }
                }
                Err(e) => eprintln!("Lua Execution Failed: {}", e),
            }
        });
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
