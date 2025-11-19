mod app;
mod core;
mod interop;
mod scripting;
mod tools;
mod ui;

use crate::app::App;
use crate::core::config::{Commands, Config};
use std::env;
use std::path::PathBuf;

fn main() {
    ui::catalog::init();

    let config = Config::parse();

    if let Some(Commands::Init { dir }) = config.command {
        tools::generator::generate(dir).unwrap();
        return;
    }

    let target_file = config.file.or_else(|| {
        let home = env::var("HOME").ok()?;
        let default_path = PathBuf::from(home)
            .join(".config")
            .join("ink")
            .join("main.lua");

        if default_path.exists() {
            println!("Loading default config: {:?}", default_path);
            Some(default_path)
        } else {
            None
        }
    });

    if let Some(file) = target_file {
        let app = App::new(file, config.windowed);
        app.run();
    } else {
        eprintln!("Error: No file provided.");
        eprintln!("Usage: ink <file.lua>");
        eprintln!("   Or: ink init (to create ~/.config/ink/main.lua)");
    }
}
