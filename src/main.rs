mod app;
mod core;
mod interop;
mod scripting;
mod services;
mod tools;
mod ui;
use crate::app::App;
use crate::core::config::{Commands, Config};
use crate::core::context::AppContext;
use crate::ui::builder::UiBuilder;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    ui::catalog::init();
    let config = Config::parse();
    if let Some(Commands::Init { dir }) = config.command {
        if let Err(e) = tools::generator::generate(dir) {
            eprintln!("Error generating files: {}", e);
            std::process::exit(1);
        }
        return;
    }
    let target_file = if config.file.is_some() {
        config.file
    } else {
        let home_dir = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let default_path = PathBuf::from(home_dir)
            .join(".config")
            .join("ink")
            .join("init.lua");

        if default_path.exists() {
            Some(default_path)
        } else {
            eprintln!("Default config not found at: {:?}", default_path);
            None
        }
    };
    if let Some(file) = target_file {
        let app = gtk4::Application::builder()
            .application_id("dev.ink.ui")
            .build();
        let lua = Rc::new(mlua::Lua::new());
        let ui_builder = Rc::new(RefCell::new(UiBuilder::new(lua.clone())));
        let context = AppContext::new(file.clone());
        let app_instance_context = Arc::new(context);

        if let Err(e) = scripting::globals::init(
            lua.clone(),
            app.clone(),
            app_instance_context.clone(),
            ui_builder.clone(),
        ) {
            eprintln!("Failed to initialize globals: {}", e);
            std::process::exit(1);
        };

        let mut app_instance = App::new(
            app.clone(),
            lua.clone(),
            Arc::try_unwrap(app_instance_context).unwrap_or_else(|arc| (*arc).clone()),
            config.windowed,
            ui_builder.clone(),
        );
        if let Err(e) = app_instance.setup() {
            eprintln!("Error during application setup: {}", e);
            std::process::exit(1);
        }
        app.run_with_args::<&str>(&[]);
    } else {
        eprintln!("Error: No file provided.");
        eprintln!("Usage: ink <file.lua>");
        eprintln!("   Or: ink init (to create ~/.config/ink/init.lua)");
    }
}
