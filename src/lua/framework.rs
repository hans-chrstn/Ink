use std::{fs, path::Path, sync::mpsc::channel, thread};

use gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::Application;
use mlua::{Lua, Result as LuaResult};
use notify::{RecommendedWatcher, Watcher};

use crate::lua::api;

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
        api::register_lua_api(&self.lua, &self.app)
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
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("lua") {
                println!("Loading widget: {:?}", path);
                if let Err(e) = self.load_widget_file(&path) {
                    eprintln!("Failed to load {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    pub fn watch_and_reload<P: AsRef<Path> + Send + 'static>(&self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = file_path.as_ref().to_path_buf();
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        println!("File event: {:?}", event)
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            }
        });

        Ok(())
    }

    pub fn run(&self, file_to_load: Option<String>) {
        let app = self.app.clone();
        let lua = self.lua.clone();

        app.connect_activate(move |_| {
            if let Some(file) = &file_to_load {
                if let Err(e) = lua.load(&std::fs::read_to_string(file).unwrap()).exec() {
                    eprintln!("Lua error: {}", e);
                }
            }
        });
        app.run();
    }
}
