use gtk4::prelude::*;
use gtk4::{glib, Widget};
use mlua::{Lua, Result as LuaResult, Value};

pub fn lua_value_to_gvalue(value: Value) -> LuaResult<glib::Value> {
    let gvalue = match value {
        Value::String(s) => s.to_str()?.to_value(),
        Value::Integer(i) => (i as i32).to_value(),
        Value::Number(n) => (n as f64).to_value(),
        Value::Boolean(b) => b.to_value(),
        _ => return Err(mlua::Error::RuntimeError("Unsupported value type".to_string()))
    };

    Ok(gvalue)
}

pub fn gvalue_to_lua_value(lua: &Lua, gvalue: glib::Value) -> LuaResult<Value> {
    if let Ok(s) = gvalue.get::<String>() {
        Ok(Value::String(lua.create_string(&s)?))
    } else if let Ok(i) = gvalue.get::<i32>() {
        Ok(Value::Integer(i as i64))
    } else if let Ok(f) = gvalue.get::<f64>() {
        Ok(Value::Number(f))
    } else if let Ok(b) = gvalue.get::<bool>() {
        Ok(Value::Boolean(b))
    } else {
        Ok(Value::Nil)
    }
}

pub fn add_child_to_container(container: &Widget, child: &Widget) -> LuaResult<()> {
    if container.is::<gtk4::ApplicationWindow>() {
        let window = container.downcast_ref::<gtk4::ApplicationWindow>().unwrap();
        window.set_child(Some(child));
    } else if container.is::<gtk4::Window>() {
        let window = container.downcast_ref::<gtk4::Window>().unwrap();
        window.set_child(Some(child));
    } else if container.is::<gtk4::Box>() {
        let gtk_box = container.downcast_ref::<gtk4::Box>().unwrap();
        gtk_box.append(child);
    } else if container.is::<gtk4::Grid>() {
        let grid = container.downcast_ref::<gtk4::Grid>().unwrap();
        grid.attach(child, 0, 0, 1, 1); // Default position
    } else if container.is::<gtk4::Button>() {
        let button = container.downcast_ref::<gtk4::Button>().unwrap();
        button.set_child(Some(child));
    } else if container.is::<gtk4::Frame>() {
        let frame = container.downcast_ref::<gtk4::Frame>().unwrap();
        frame.set_child(Some(child));
    } else if container.is::<gtk4::ScrolledWindow>() {
        let scrolled = container.downcast_ref::<gtk4::ScrolledWindow>().unwrap();
        scrolled.set_child(Some(child));
    } else if container.is::<gtk4::Expander>() {
        let expander = container.downcast_ref::<gtk4::Expander>().unwrap();
        expander.set_child(Some(child));
    } else if container.is::<gtk4::Notebook>() {
        let notebook = container.downcast_ref::<gtk4::Notebook>().unwrap();
        notebook.append_page(child, None::<&Widget>);
    } else {
        return Err(mlua::Error::RuntimeError(format!(
            "Cannot add child to widget of type: {}", 
            container.type_().name()
        )));
    }
    Ok(())
}

pub fn is_layer_shell_compatible(widget: &Widget) -> bool {
    widget.is::<gtk4::ApplicationWindow>() || widget.is::<gtk4::Window>()
}
