use crate::interop::{converter::GenericConverter, signals::SignalConnector};
use crate::scripting::traits::ScriptValue;
use crate::ui::registry::Registry;
use crate::ui::traits::WidgetBehavior;
use gtk4::Widget;
use gtk4::glib::{GString, Object};
use gtk4::prelude::*;
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::scripting::globals::get_core_context;
use crate::scripting::lua_driver::LuaWrapper;
use crate::scripting::widget_wrapper::LuaWidget as WidgetWrapper;

const ALLOWED_TOP_LEVEL_KEYS: &[&str] = &[
    "id",
    "type",
    "properties",
    "signals",
    "children",
    "window_mode",
    "layer",
    "anchors",
    "margins",
    "auto_exclusive_zone",
    "keyboard_mode",
    "css",
    "css_path",
    "realize",
    "actions",
    "menu",
    "draw",
    "keymaps",
];

pub struct UiBuilder {
    behaviors: HashMap<String, Box<dyn WidgetBehavior<LuaWrapper>>>,
    lua: Rc<Lua>,
    widgets_by_id: RefCell<HashMap<String, Widget>>,
}

impl UiBuilder {
    pub fn new(lua: Rc<Lua>) -> Self {
        Self {
            behaviors: HashMap::new(),
            lua,
            widgets_by_id: RefCell::new(HashMap::new()),
        }
    }

    pub fn register_behavior(
        &mut self,
        type_name: &str,
        behavior: Box<dyn WidgetBehavior<LuaWrapper>>,
    ) -> &mut Self {
        self.behaviors.insert(type_name.to_string(), behavior);
        self
    }

    pub fn build(&self, data: &LuaWrapper, config_dir: &Path) -> Result<Widget, String> {
        let root_widget = self.build_recursive(data, config_dir)?;
        Ok(root_widget)
    }

    pub fn get_widget_by_id(&self, id: &str) -> Option<Widget> {
        let widget = self.widgets_by_id.borrow().get(id).cloned();

        widget
    }

    pub fn register_get_widget_by_id_lua_function(
        lua: &Rc<Lua>,
        app_global: &Table,
    ) -> mlua::Result<()> {
        app_global.set(
            "get_widget_by_id",
            lua.create_function(|lua, id: String| {
                let core_context = get_core_context(lua)?;
                let core_context_guard = core_context.borrow();
                let ui_builder_guard = core_context_guard.ui_builder.borrow();
                let widget = ui_builder_guard
                    .get_widget_by_id(&id)
                    .ok_or_else(|| mlua::Error::runtime(format!("Widget not found: {}", id)))?;
                Ok(WidgetWrapper(widget))
            })?,
        )?;
        Ok(())
    }

    fn build_recursive(&self, data: &LuaWrapper, config_dir: &Path) -> Result<Widget, String> {
        let (widget, type_name) = self.instantiate_widget(data)?;

        self.register_widget_id(data, &widget);

        self.set_widget_properties(data, &widget, &type_name, config_dir)?;

        self.connect_widget_signals(data, &widget);

        self.add_widget_children(data, &widget, &type_name, config_dir)?;

        self.apply_widget_behavior(&widget, &type_name, data);

        if let Some(entries) = data.get_map_entries() {
            for (key, _) in entries {
                if !ALLOWED_TOP_LEVEL_KEYS.contains(&key.as_str()) {
                    return Err(format!(
                        "Unknown top-level configuration property: '{}' for widget type '{}'. \
                        Did you mean to put it inside 'properties = {{}}'?",
                        key, type_name
                    ));
                }
            }
        }

        Ok(widget)
    }

    fn instantiate_widget(&self, data: &LuaWrapper) -> Result<(Widget, String), String> {
        let type_name = data
            .get_property("type")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "GtkWindow".to_string());

        let gtype =
            Registry::get_type(&type_name).ok_or_else(|| format!("Unknown type: {}", type_name))?;

        let object = Object::with_type(gtype);
        let widget = object
            .downcast::<Widget>()
            .map_err(|_| "Not a widget".to_string())?;

        Ok((widget, type_name))
    }

    fn register_widget_id(&self, data: &LuaWrapper, widget: &Widget) {
        if let Some(id_val) = data.get_property("id")
            && let Some(id_str) = id_val.as_string()
        {
            self.widgets_by_id
                .borrow_mut()
                .insert(id_str, widget.clone());
        };
    }

    fn set_widget_properties(
        &self,
        data: &LuaWrapper,
        widget: &Widget,
        type_name: &str,
        config_dir: &Path,
    ) -> Result<(), String> {
        if let Some(props) = data
            .get_property("properties")
            .and_then(|v| v.get_map_entries())
        {
            const CONTAINER_PROPERTIES: &[&str] =
                &["grid_col", "grid_row", "grid_width", "grid_height"];

            for (k, v) in props {
                let gtk_property_name = k.replace("_", "-");

                if CONTAINER_PROPERTIES.contains(&k.as_str()) {
                    continue;
                }

                if let Some(pspec) = widget.find_property(&gtk_property_name) {
                    let is_path_prop = (k == "file" || k == "icon-name" || k == "file-name")
                        && pspec.value_type() == GString::static_type();
                    if is_path_prop {
                        if let Some(path_str) = v.as_string() {
                            let path = Path::new(&path_str);
                            let final_path = if path.is_absolute() {
                                path.to_path_buf()
                            } else {
                                config_dir.join(path)
                            };

                            let lua_string = self
                                .lua
                                .create_string(&*final_path.to_string_lossy())
                                .map_err(|e| format!("Failed to create Lua string: {}", e))?;
                            let resolved_path_wrapper = LuaWrapper(mlua::Value::String(lua_string));

                            let gval = GenericConverter::to_gvalue(
                                &resolved_path_wrapper,
                                pspec.value_type(),
                            )
                            .map_err(|e| format!("Failed to convert path property: {}", e))?;
                            widget.set_property(&gtk_property_name, gval);
                        } else {
                            return Err(format!(
                                "Property '{}' on type '{}' expects a string, but got non-string value",
                                k, type_name
                            ));
                        }
                    } else {
                        let gval = GenericConverter::to_gvalue(&v, pspec.value_type())
                            .map_err(|e| format!("Failed to convert property: {}", e))?;
                        widget.set_property(&gtk_property_name, gval);
                    }
                } else {
                    return Err(format!(
                        "Property '{}' not found on type '{}'",
                        k, type_name
                    ));
                }
            }
        }
        Ok(())
    }

    fn connect_widget_signals(&self, data: &LuaWrapper, widget: &Widget) {
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
    }

    fn add_widget_children(
        &self,
        data: &LuaWrapper,
        widget: &Widget,
        type_name: &str,
        config_dir: &Path,
    ) -> Result<(), String> {
        if let Some(children) = data
            .get_property("children")
            .and_then(|v| v.get_array_items())
        {
            let strategy = Registry::get_strategy(type_name, widget);
            for child_data in children {
                let child_widget = self.build_recursive(&child_data, config_dir)?;
                strategy.add_child(&child_widget, &child_data)?;
            }
        }
        Ok(())
    }

    fn apply_widget_behavior(&self, widget: &Widget, type_name: &str, data: &LuaWrapper) {
        if let Some(behavior) = self.behaviors.get(type_name) {
            behavior.apply(widget, data);
        }
    }
}
