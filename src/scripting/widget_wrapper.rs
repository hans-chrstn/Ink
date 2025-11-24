use crate::scripting::lua_driver::LuaWrapper;
use crate::ui::registry::Registry;
use gtk4::glib::prelude::*;
use gtk4::glib::Type as GType;
use gtk4::prelude::*;
use mlua::{Error, FromLua, Function, Lua, UserData, UserDataMethods, Value};
#[derive(Clone, Copy)]
pub struct LuaGType(pub GType);
impl UserData for LuaGType {}
impl FromLua for LuaGType {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let ud = value
            .as_userdata()
            .ok_or_else(|| Error::FromLuaConversionError {
                from: value.type_name(),
                to: "GType".to_string(),
                message: Some("Expected a GType object".to_string()),
            })?;
        let w = ud.borrow::<Self>()?;
        Ok(*w)
    }
}
#[derive(Clone)]
pub struct LuaWidget(pub gtk4::Widget);
impl FromLua for LuaWidget {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let ud = value
            .as_userdata()
            .ok_or_else(|| Error::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaWidget".to_string(),
                message: Some("Expected a Widget object".to_string()),
            })?;
        let w = ud.borrow::<Self>()?;
        Ok(w.clone())
    }
}
fn find_child_by_name_recursive(parent: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
    let mut child = parent.first_child();
    while let Some(c) = child {
        if c.widget_name() == name {
            return Some(c);
        }
        if let Some(found) = find_child_by_name_recursive(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
impl UserData for LuaWidget {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("destroy", |_, this, ()| {
            if let Some(window) = this.0.downcast_ref::<gtk4::Window>() {
                window.destroy();
            } else {
                this.0.unparent();
            }
            Ok(())
        });
        methods.add_method("get_ancestor", |_, this, gtype: LuaGType| {
            let ancestor = this.0.ancestor(gtype.0);
            if let Some(ancestor) = ancestor {
                Ok(Some(LuaWidget(ancestor)))
            } else {
                Ok(None)
            }
        });
        methods.add_method("find_child", |_, this, name: String| {
            if let Some(child) = find_child_by_name_recursive(&this.0, &name) {
                Ok(Some(LuaWidget(child)))
            } else {
                Ok(None)
            }
        });
        methods.add_method("set_text", |_, this, text: String| {
            if let Some(editable) = this.0.downcast_ref::<gtk4::Editable>() {
                editable.set_text(&text);
            } else if let Some(label) = this.0.downcast_ref::<gtk4::Label>() {
                label.set_text(&text);
            }
            Ok(())
        });
        methods.add_method("insert_text", |_, this, text: String| {
            if let Some(editable) = this.0.downcast_ref::<gtk4::Editable>() {
                editable.insert_text(&text, &mut editable.position());
            }
            Ok(())
        });
        methods.add_method("set_visible", |_, this, visible: bool| {
            this.0.set_visible(visible);
            Ok(())
        });
        methods.add_method("add_class", |_, this, class: String| {
            this.0.add_css_class(&class);
            Ok(())
        });
        methods.add_method("remove_class", |_, this, class: String| {
            this.0.remove_css_class(&class);
            Ok(())
        });
        methods.add_method("remove_children", |_, this, ()| {
            if let Some(flowbox) = this.0.downcast_ref::<gtk4::FlowBox>() {
                while let Some(child) = flowbox.first_child() {
                    flowbox.remove(&child);
                }
            } else {
                let mut children = Vec::new();
                let mut child = this.0.first_child();
                while let Some(c) = child {
                    children.push(c.clone());
                    child = c.next_sibling();
                }
                for c in children {
                    c.unparent();
                }
            }
            Ok(())
        });
        methods.add_method("grab_focus", |_, this, ()| {
            this.0.grab_focus();
            Ok(())
        });
        methods.add_method("get_text", |_, this, ()| {
            if let Some(editable) = this.0.downcast_ref::<gtk4::Editable>() {
                return Ok(editable.text().to_string());
            }
            Ok("".to_string())
        });
        methods.add_method("get_value", |_, this, ()| {
            if let Some(range) = this.0.downcast_ref::<gtk4::Range>() {
                return Ok(range.value());
            }
            if let Some(pb) = this.0.downcast_ref::<gtk4::ProgressBar>() {
                return Ok(pb.fraction());
            }
            Ok(0.0)
        });
        methods.add_method("set_value", |_, this, val: f64| {
            if let Some(range) = this.0.downcast_ref::<gtk4::Range>() {
                range.set_value(val);
            } else if let Some(pb) = this.0.downcast_ref::<gtk4::ProgressBar>() {
                pb.set_fraction(val);
            }
            Ok(())
        });
        methods.add_method("set_range", |_, this, (min, max): (f64, f64)| {
            if let Some(range) = this.0.downcast_ref::<gtk4::Range>() {
                range.set_range(min, max);
            }
            Ok(())
        });
        methods.add_method("set_increments", |_, this, (step, page): (f64, f64)| {
            if let Some(range) = this.0.downcast_ref::<gtk4::Range>() {
                range.set_increments(step, page);
            }
            Ok(())
        });
        methods.add_method(
            "add_controller_motion",
            |lua, this, (on_enter, on_leave): (Function, Function)| {
                let controller = gtk4::EventControllerMotion::new();
                let enter_cb = lua.create_registry_value(on_enter)?;
                let lua_enter = lua.clone();
                controller.connect_enter(move |_, _, _| {
                    if let Ok(func) = lua_enter.registry_value::<Function>(&enter_cb) {
                        let _ = func.call::<()>(());
                    }
                });
                let leave_cb = lua.create_registry_value(on_leave)?;
                let lua_leave = lua.clone();
                controller.connect_leave(move |_| {
                    if let Ok(func) = lua_leave.registry_value::<Function>(&leave_cb) {
                        let _ = func.call::<()>(());
                    }
                });
                this.0.add_controller(controller);
                Ok(())
            },
        );
        methods.add_method("add", |_, this, (child, props): (LuaWidget, Value)| {
            let type_name = this.0.type_().name();
            let strategy = Registry::get_strategy(type_name, &this.0);
            let wrapper = LuaWrapper(props);
            strategy.add_child(&child.0, &wrapper);
            Ok(())
        });
        methods.add_method("set_property", |_, this, (key, val): (String, Value)| {
            match val {
                Value::String(s) => {
                    if let Ok(rust_str) = s.to_str() {
                        this.0.set_property(&key, rust_str.as_ref());
                    }
                }
                Value::Boolean(b) => this.0.set_property(&key, b),
                Value::Number(n) => this.0.set_property(&key, n),
                Value::Integer(i) => this.0.set_property(&key, i as f64),
                _ => {}
            }
            Ok(())
        });
    }
}
