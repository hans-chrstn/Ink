use std::{env, path::Path};

use gtk4::{gdk::Display, prelude::*, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4_layer_shell::*;
use rsass::{compile_scss_path, output};

use crate::lua::WidgetFramework;

const APP_ID: &str = "dev.mishima.ink";
mod lua;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    gtk4::init().expect("Failed to initialize GTK");

    let lua_framework = WidgetFramework::new("dev.hans.ink");

    lua_framework.register_api()?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <widget.lua> or {} --dir <widgets_directors>", args[0], args[0]);
        eprintln!("Example: {} examples/clock.lua", args[0]);
        eprintln!("Example: {} --dir widgets/", args[0]);
        std::process::exit(1);
    }

    if args[1] == "--dir" && args.len() >= 3 {
        lua_framework.load_widget_from_dir(&args[2])?;
    } else {
        lua_framework.load_widget_file(&args[1])?;
    }

    if env::var("WATCH").is_ok() {
        println!("File watching enabled. Set WATCH=1 to enable hot reload.");
        if args.len() >= 2 && args[1] != "--dir" {
            let file_to_watch = args[1].clone();
            lua_framework.watch_and_reload(file_to_watch)?;
        }
    }

    lua_framework.run();
    Ok(())
}

pub fn load_css(path: &str) {
    let path = Path::new(path);
    let css_bytes = compile_scss_path(path, output::Format::default()).expect("SCSS Compilation failed");
    let css_str = std::str::from_utf8(&css_bytes).expect("Compiled CSS is not a valid UTF-8");
    let provider = CssProvider::new();
    provider.load_from_data(css_str);

    style_context_add_provider_for_display(
        &Display::default().expect("No display"),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
