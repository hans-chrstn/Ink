<<<<<<< HEAD
=======
use crate::{app::App, config::Config};
>>>>>>> 24d3837 ((fix): updated HEAD to latest commit (main))
use gtk4::{
    CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk::Display,
    style_context_add_provider_for_display,
};
use rsass::{compile_scss_path, output};
use std::path::Path;

<<<<<<< HEAD
use crate::{app::App, config::Config};

=======
>>>>>>> 24d3837 ((fix): updated HEAD to latest commit (main))
mod app;
mod config;
mod error;
mod register;
mod ui;

fn main() {
    let app = App::new();
    app.connect_signals();
    app.run();
}

pub fn load_css(path: &str) {
    let path = Path::new(path);
    let css_bytes =
        compile_scss_path(path, output::Format::default()).expect("SCSS Compilation failed");
    let css_str = std::str::from_utf8(&css_bytes).expect("Compiled CSS is not a valid UTF-8");
    let provider = CssProvider::new();
    provider.load_from_data(css_str);

    style_context_add_provider_for_display(
        &Display::default().expect("No display"),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
