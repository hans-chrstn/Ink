use std::{env, path::Path};

use gtk4::{gdk::Display, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use rsass::{compile_scss_path, output};

use crate::lua::WidgetFramework;

const APP_ID: &str = "dev.mishima.ink";
mod lua;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    gtk4::init().expect("Failed to initialize GTK");

    let lua_framework = WidgetFramework::new(APP_ID);

    lua_framework.register_api()?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <widget.lua> or {} --dir <widgets_directors>", args[0], args[0]);
        eprintln!("Example: {} examples/clock.lua", args[0]);
        eprintln!("Example: {} --dir widgets/", args[0]);
        std::process::exit(1);
    }

    let file_to_load = if args[1] == "--dir" && args.len() >= 3 {
        Some(args[2].clone())
    } else {
        Some(args[1].clone())
    };


    lua_framework.run(file_to_load);
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
