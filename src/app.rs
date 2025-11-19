use crate::scripting::globals;
use crate::scripting::lua_driver::LuaWrapper;
use crate::ui::builder::UiBuilder;
use crate::ui::strategy::WindowStrategy;
use gtk4::{Application, prelude::*};
use mlua::{Lua, Table};
use std::path::PathBuf;
use std::rc::Rc;

pub struct App {
    app: Application,
    lua: Rc<Lua>,
    file: PathBuf,
    windowed: bool,
}

impl App {
    pub fn new(file: PathBuf, windowed: bool) -> Self {
        let app = Application::builder().application_id("dev.ink.ui").build();
        let lua = Rc::new(Lua::new());

        globals::init(&lua).expect("Failed to initialize Lua globals");

        Self {
            app,
            lua,
            file,
            windowed,
        }
    }

    pub fn run(&self) {
        let lua = self.lua.clone();
        let path = self.file.clone();
        let windowed = self.windowed;

        self.app.connect_activate(move |app| {
            if let Some(parent) = path.parent().and_then(|p| p.to_str()) {
                let _ = lua.globals().get::<Table>("package").unwrap().set(
                    "path",
                    format!("{};{}/?.lua;{}/?/init.lua", "", parent, parent),
                );
            }

            if !path.exists() {
                eprintln!("Error: File not found: {:?}", path);
                return;
            }

            let code = std::fs::read_to_string(&path).expect("Read failed");

            match lua.load(&code).call::<mlua::Value>(()) {
                Ok(table_val) => {
                    if let mlua::Value::Table(table) = table_val {
                        let wrapped = LuaWrapper(mlua::Value::Table(table));

                        let builder = UiBuilder::<LuaWrapper>::new().register_behavior(
                            "GtkApplicationWindow",
                            Box::new(WindowStrategy::new(windowed)),
                        );

                        match builder.build(&wrapped) {
                            Ok(root) => {
                                if let Some(w) = root.downcast_ref::<gtk4::Window>() {
                                    w.set_application(Some(app));
                                }
                            }
                            Err(e) => eprintln!("Builder Error: {}", e),
                        }
                    } else {
                        eprintln!("Error: Lua script must return a UI Table (WidgetConfig)");
                    }
                }
                Err(e) => eprintln!("Lua Execution Failed: {}", e),
            }
        });

        self.app.run_with_args::<&str>(&[]);
    }
}
