use crate::interop::{converter::GenericConverter, signals::SignalConnector};
use crate::scripting::traits::ScriptValue;
use crate::ui::registry::Registry;
use crate::ui::traits::WidgetBehavior;
use gtk4::Widget;
use gtk4::glib::Object;
use gtk4::prelude::*;
use std::collections::HashMap;

use crate::scripting::lua_driver::LuaWrapper;

pub struct UiBuilder {
    behaviors: HashMap<String, Box<dyn WidgetBehavior<LuaWrapper>>>,
}

impl UiBuilder {
    pub fn new() -> Self {
        Self {
            behaviors: HashMap::new(),
        }
    }

    pub fn register_behavior(
        mut self,
        type_name: &str,
        behavior: Box<dyn WidgetBehavior<LuaWrapper>>,
    ) -> Self {
        self.behaviors.insert(type_name.to_string(), behavior);
        self
    }

    pub fn build(&self, data: &LuaWrapper) -> Result<Widget, String> {
        self.build_recursive(data)
    }

    fn build_recursive(&self, data: &LuaWrapper) -> Result<Widget, String> {
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
                let child_widget = self.build_recursive(&child_data)?;
                strategy.add_child(&child_widget, &child_data);
            }
        }

        if let Some(behavior) = self.behaviors.get(&type_name) {
            behavior.apply(&widget, data);
        }

        Ok(widget)
    }
}
