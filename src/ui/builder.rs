use crate::interop::{converter::GenericConverter, signals::SignalConnector};
use crate::scripting::traits::ScriptValue;
use crate::ui::registry::Registry;
use crate::ui::strategy::WindowStrategy;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Widget, glib::Object};

pub struct UiBuilder;

impl UiBuilder {
    pub fn run<T: ScriptValue + 'static>(data: &T, force_windowed: bool) -> Result<Widget, String> {
        let root = Self::build_recursive(data)?;

        if let Some(win) = root.downcast_ref::<ApplicationWindow>() {
            WindowStrategy::apply(win, data, force_windowed);
        }

        Ok(root)
    }

    fn build_recursive<T: ScriptValue + 'static>(data: &T) -> Result<Widget, String> {
        let type_name = data
            .get_property("type")
            .and_then(|v| v.as_string())
            .ok_or("Widget missing 'type' field")?;

        let gtype =
            Registry::get_type(&type_name).ok_or_else(|| format!("Unknown type: {}", type_name))?;

        let object = Object::with_type(gtype);
        let widget = object
            .downcast::<Widget>()
            .map_err(|_| "Not a widget".to_string())?;

        if let Some(props) = data
            .get_property("properties")
            .and_then(|v| v.get_map_entries())
        {
            for (k, v) in props {
                if let Some(pspec) = widget.find_property(&k) {
                    if let Some(gval) = GenericConverter::to_gvalue(&v, pspec.value_type()) {
                        widget.set_property(&k, gval);
                    }
                }
            }
        }

        if let Some(sigs) = data
            .get_property("signals")
            .and_then(|v| v.get_map_entries())
        {
            for (name, func) in sigs {
                if func.is_function() {
                    SignalConnector::connect(widget.upcast_ref(), &name, func);
                }
            }
        }

        if let Some(children) = data
            .get_property("children")
            .and_then(|v| v.get_array_items())
        {
            let strategy = Registry::get_strategy(&type_name, &widget);
            for child_data in children {
                let child_widget = Self::build_recursive(&child_data)?;
                strategy.add_child(&child_widget);
            }
        }

        Ok(widget)
    }
}
