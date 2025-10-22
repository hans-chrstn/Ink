use std::rc::Rc;

use gio::ApplicationFlags;
use gtk4::{Application, prelude::*};
use mlua::Lua;

use crate::{Config, register, ui};

const APP_ID: &str = "dev.hans.ink";

pub struct App {
    app: Application,
    lua: Rc<Lua>,
    config: Config,
}

impl App {
    pub fn new() -> Self {
        let config = Config::parse();
        let app = Application::builder()
            .application_id(APP_ID)
            .flags(ApplicationFlags::HANDLES_OPEN)
            .build();
        let lua = Rc::new(Lua::new());
        register::register_widgets();
        Self { app, lua, config }
    }

    pub fn connect_signals(&self) {
        self.app.connect_activate(|_| {
            eprintln!("This application requires a Lua file to build a UI.");
            eprintln!("Usage: ink <PATH_TO_LUA_FILE>");
        });

        let lua = self.lua.clone();
        let config = self.config.clone();

        self.app.connect_open(move |app, files, _| {
            if let Some(file) = files.get(0) {
                let file_path = file.path().expect("Couldn't get file path");
                match ui::build_from_file(app, &lua, file_path.to_str().unwrap(), &config) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error building UI: {}", e),
                }
            }
        });
    }

    pub fn run(&self) {
        self.app.run();
    }
}
