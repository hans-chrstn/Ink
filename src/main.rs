use std::path::Path;

use gtk4::{gdk::Display, prelude::*, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4_layer_shell::*;
use rsass::{compile_scss_path, output};

const APP_ID: &str = "dev.mishima.ink";
fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
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

pub fn build_ui(app: &Application) {
    // load scss
    load_css("src/styles/main.scss");

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(300)
        .default_height(50)
        .resizable(false)
        .decorated(false)
        .title("Ink")
        .build();

    window.init_layer_shell();

    window.set_layer(Layer::Overlay);

    window.auto_exclusive_zone_enable();

    let anchors = [
        (Edge::Left, false),
        (Edge::Right, false),
        (Edge::Top, true),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }

    // Present window
    window.present();
}
