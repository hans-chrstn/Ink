use crate::scripting::traits::ScriptValue;
use crate::ui::traits::{WidgetContainer, WidgetBehavior};
use gtk4::{ApplicationWindow, Widget, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct WindowStrategy {
    pub force_windowed: bool,
}

impl WindowStrategy {
    pub fn new(force_windowed: bool) -> Self {
        Self { force_windowed }
    }

    fn set_anchor<T: ScriptValue>(w: &ApplicationWindow, data: &T, key: &str, edge: Edge) {
        if let Some(val) = data.get_property(key).and_then(|v| v.as_bool()) {
            w.set_anchor(edge, val);
        }
    }
}

impl<T: ScriptValue> WidgetBehavior<T> for WindowStrategy {
    fn apply(&self, widget: &Widget, data: &T) {
        let Some(window) = widget.downcast_ref::<ApplicationWindow>() else { return };

        if self.force_windowed {
            window.present();
            return;
        }

        let mode = data.get_property("window_mode").and_then(|v| v.as_string()).unwrap_or_else(|| "layer_shell".into());

        if mode == "normal" {
            window.present();
            return;
        }

        window.init_layer_shell();
        window.present();

        let layer = match data.get_property("layer").and_then(|v| v.as_string()).as_deref() {
            Some("bottom") => Layer::Bottom,
            Some("overlay") => Layer::Overlay,
            Some("background") => Layer::Background,
            _ => Layer::Top,
        };
        window.set_layer(layer);

        if let Some(anchors) = data.get_property("anchors") {
            Self::set_anchor(window, &anchors, "top", Edge::Top);
            Self::set_anchor(window, &anchors, "bottom", Edge::Bottom);
            Self::set_anchor(window, &anchors, "left", Edge::Left);
            Self::set_anchor(window, &anchors, "right", Edge::Right);
        } else {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
        }

        if let Some(z) = data.get_property("exclusive_zone").and_then(|v| v.as_integer()) {
            window.set_exclusive_zone(z as i32);
        } else if let Some(true) = data.get_property("auto_exclusive_zone").and_then(|v| v.as_bool()) {
            window.auto_exclusive_zone_enable();
        }
        
        if let Some(kb) = data.get_property("keyboard_mode").and_then(|v| v.as_string()) {
             let mode = match kb.as_str() {
                 "exclusive" => gtk4_layer_shell::KeyboardMode::Exclusive,
                 "on_demand" => gtk4_layer_shell::KeyboardMode::OnDemand,
                 _ => gtk4_layer_shell::KeyboardMode::None,
             };
             window.set_keyboard_mode(mode);
        }
    }
}

#[derive(Clone)]
pub struct GridStrategy {
    grid: gtk4::Grid,
}

impl GridStrategy {
    pub fn new(grid: gtk4::Grid) -> Self {
        Self { grid }
    }
}

impl<T: ScriptValue> WidgetContainer<T> for GridStrategy {
    fn add_child(&self, child: &Widget, data: &T) {
        let col = data.get_property("grid_col").and_then(|v| v.as_integer()).unwrap_or(0) as i32;
        let row = data.get_property("grid_row").and_then(|v| v.as_integer()).unwrap_or(0) as i32;
        let w = data.get_property("grid_width").and_then(|v| v.as_integer()).unwrap_or(1) as i32;
        let h = data.get_property("grid_height").and_then(|v| v.as_integer()).unwrap_or(1) as i32;
        
        self.grid.attach(child, col, row, w, h);
    }
}
