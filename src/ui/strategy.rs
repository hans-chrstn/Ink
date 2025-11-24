use crate::scripting::traits::ScriptValue;
use crate::ui::traits::{WidgetBehavior, WidgetContainer};
use gtk4::prelude::*;
use gtk4::Widget;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
pub struct WindowStrategy {
    pub force_windowed: bool,
}
impl WindowStrategy {
    pub fn new(force_windowed: bool) -> Self {
        Self { force_windowed }
    }
    fn set_anchor<T: ScriptValue>(w: &gtk4::Window, data: &T, key: &str, edge: Edge) {
        if let Some(val) = data.get_property(key).and_then(|v| v.as_bool()) {
            w.set_anchor(edge, val);
        }
    }
}
impl<T: ScriptValue + 'static> WidgetBehavior<T> for WindowStrategy {
    fn apply(&self, widget: &Widget, data: &T) {
        let Some(window) = widget.downcast_ref::<gtk4::Window>() else {
            return;
        };
        if self.force_windowed {
            window.present();
            return;
        }
        if let Some(maps) = data
            .get_property("keymaps")
            .and_then(|v| v.get_map_entries())
        {
            let controller = gtk4::EventControllerKey::new();
            controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
            let mut bindings = Vec::new();
            for (key_str, func) in maps {
                if let Some((keyval, mods)) = gtk4::accelerator_parse(&key_str) {
                    bindings.push((keyval, mods, func));
                } else {
                    eprintln!("Warn: Invalid keybind string '{}'", key_str);
                }
            }
            controller.connect_key_pressed(move |_, keyval, _keycode, state| {
                for (bind_key, bind_mods, func) in &bindings {
                    if keyval == *bind_key && state.contains(*bind_mods) {
                        if let Err(e) = func.call(vec![]) {
                            eprintln!("Keybind Error: {}", e);
                        }
                        return gtk4::glib::Propagation::Stop;
                    }
                }
                gtk4::glib::Propagation::Proceed
            });
            window.add_controller(controller);
        }
        let mode = data
            .get_property("window_mode")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "layer_shell".into());
        if mode == "normal" {
            window.present();
            return;
        }
        window.init_layer_shell();
        window.present();
        let layer = match data
            .get_property("layer")
            .and_then(|v| v.as_string())
            .as_deref()
        {
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
        if let Some(z) = data
            .get_property("exclusive_zone")
            .and_then(|v| v.as_integer())
        {
            window.set_exclusive_zone(z as i32);
        } else if let Some(true) = data
            .get_property("auto_exclusive_zone")
            .and_then(|v| v.as_bool())
        {
            window.auto_exclusive_zone_enable();
        }
        let kb_mode = data
            .get_property("keyboard_mode")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "on_demand".to_string());
        let mode = match kb_mode.as_str() {
            "exclusive" => gtk4_layer_shell::KeyboardMode::Exclusive,
            "none" => gtk4_layer_shell::KeyboardMode::None,
            _ => gtk4_layer_shell::KeyboardMode::OnDemand,
        };
        window.set_keyboard_mode(mode);
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
        let props = data.get_property("properties");
        let get_val = |key: &str, default: i32| -> i32 {
            if let Some(v) = data.get_property(key).and_then(|x| x.as_integer()) {
                return v as i32;
            }
            if let Some(p) = &props {
                if let Some(v) = p.get_property(key).and_then(|x| x.as_integer()) {
                    return v as i32;
                }
            }
            default
        };
        let col = get_val("grid_col", 0);
        let row = get_val("grid_row", 0);
        let w = get_val("grid_width", 1);
        let h = get_val("grid_height", 1);
        self.grid.attach(child, col, row, w, h);
    }
}
