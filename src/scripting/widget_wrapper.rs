use gtk4::prelude::*;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Clone)]
pub struct LuaWidget(pub gtk4::Widget);

impl UserData for LuaWidget {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
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
