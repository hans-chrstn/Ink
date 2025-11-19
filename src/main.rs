mod app;
mod core;
mod interop;
mod scripting;
mod tools;
mod ui;

use crate::app::App;
use crate::core::config::{Commands, Config};

fn main() {
    ui::catalog::init();

    let config = Config::parse();

    if let Some(Commands::Init { dir }) = config.command {
        tools::generator::generate(dir).unwrap();
        return;
    }

    if let Some(file) = config.file {
        let app = App::new(file, config.windowed);
        app.run();
    } else {
        eprintln!("Please provide a file: ink main.lua");
    }
}
