use std::{cell::RefCell, collections::HashMap, fs, path::Path, rc::Rc};

use gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::Application;
use mlua::{Lua, Result as LuaResult, Table, Function};

pub struct WidgetFramework {
    lua: Lua,
    app: Application,
}

impl WidgetFramework {
    pub fn new(app_id: &str) -> Self {
        let app = Application::builder()
            .application_id(app_id)
            .build();

        let lua = Lua::new();

        Self { lua, app }
    }

    pub fn register_api(&self) -> LuaResult<()> {
    }

    pub fn load_widget_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = file_path.as_ref();
        println!("Loading widget from: {:?}", path);

        let lua_code = fs::read_to_string(path)?;
        self.lua.load(&lua_code).exec()?;

        Ok(())
    }

    pub fn load_widget_string(&self, lua_code: &str) -> LuaResult<()> {
        self.lua.load(lua_code).exec()
    }

    pub fn load_widget_from_dir<P: AsRef<Path>>(&self, dir_path: P) -> Result<(), Box<dyn std::error::Error>> {

        Ok(())
    }

    pub fn watch_and_reload<P: AsRef<Path> + Send + 'static>(&self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn run(&self) {
        let app = self.app.clone();
        app.connect_activate(|_| {});
        app.run();
    }
}
