use gtk4::{
    Application, ApplicationWindow, Box, Widget,
    glib::{Object, Type, Value, object::ObjectExt, value::ToValue},
    prelude::*,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use mlua::{Lua, Result, Table, Value as LuaValue};
use std::fs;
use std::rc::Rc;

use crate::config::Config;

pub fn build_from_file(
    app: &Application,
    lua: &Rc<Lua>,
    file_path: &str,
    config: &Config,
) -> Result<()> {
    let lua_code = fs::read_to_string(file_path)?;
    let root_table: Table = lua.load(&lua_code).call(())?;
    let root_widget = create_widget(app, lua, root_table)?;

    if !config.windowed {
        if let Ok(window) = root_widget.clone().downcast::<ApplicationWindow>() {
            init_layer_shell(&window);
        }
    }

    if let Ok(window) = root_widget.downcast::<ApplicationWindow>() {
        window.present();
    } else {
        panic!("Root widget must be a window type");
    }

    Ok(())
}

fn create_widget(app: &Application, lua: &Rc<Lua>, widget_table: Table) -> Result<gtk4::Widget> {
    let type_name: String = widget_table.get("type")?;
    let props: Table = widget_table
        .get("properties")
        .unwrap_or(lua.create_table()?);

    let widget_type =
        Type::from_name(&type_name).unwrap_or_else(|| panic!("Unknown widget type: {}", type_name));

    let widget_object = Object::with_type(widget_type);

    if widget_object.is::<ApplicationWindow>() {
        widget_object.set_property("application", app);
    }

    for pair in props.pairs::<String, LuaValue>() {
        let (key, value) = pair?;
        if let Some(glib_value) = to_glib_value(value) {
            widget_object.set_property(&key, glib_value);
        }
    }

    let widget = widget_object.downcast::<Widget>().unwrap();

    if let Ok(children) = widget_table.get::<Table>("children") {
        for child_table in children.sequence_values::<Table>() {
            let child_widget = create_widget(app, lua, child_table?)?;

            if let Some(container) = widget.downcast_ref::<Box>() {
                container.append(&child_widget);
            } else if let Some(window) = widget.downcast_ref::<ApplicationWindow>() {
                window.set_child(Some(&child_widget));
            }
        }
    }
    Ok(widget)
}

fn to_glib_value(lua_value: LuaValue) -> Option<Value> {
    match lua_value {
        LuaValue::String(s) => match s.to_str() {
            Ok(valid_str) => Some(valid_str.to_value()),
            Err(_) => Some("".to_value()),
        },
        LuaValue::Integer(i) => Some((i as i32).to_value()),
        LuaValue::Number(n) => Some((n as f32).to_value()),
        LuaValue::Boolean(b) => Some(b.to_value()),
        _ => None,
    }
}

fn init_layer_shell(window: &ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.auto_exclusive_zone_enable();
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
}
