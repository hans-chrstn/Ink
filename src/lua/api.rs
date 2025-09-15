use gtk4::prelude::*;
use gtk4::{glib, Application};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use mlua::{Lua, Result as LuaResult, Table, Function, Value};

use crate::lua::widget::GtkWidget;
use crate::lua::util;

pub fn register_lua_api(lua: &Lua, app: &Application) -> LuaResult<()> {
    let globals = lua.globals();

    // Main GTK widget factory function
    let create_widget = lua.create_function(|_, (type_name, properties): (String, Option<Table>)| {
        create_gtk_widget(type_name, properties)
    })?;

    // Create gtk table with widget constructors
    let gtk_table = create_gtk_widget_table(lua, app)?;

    // Create layer shell utilities
    let layer_shell = create_layer_shell_table(lua)?;

    // Register utility functions
    register_utility_functions(lua, &globals)?;

    // Register everything
    globals.set("gtk", gtk_table)?;
    globals.set("create_widget", create_widget)?;
    globals.set("layer_shell", layer_shell)?;

    Ok(())
}

fn create_gtk_widget(type_name: String, properties: Option<Table>) -> LuaResult<GtkWidget> {
    // Get the GType for the widget
    let gtype = glib::Type::from_name(&type_name)
        .ok_or_else(|| mlua::Error::RuntimeError(format!("Unknown widget type: {}", type_name)))?;

    // Create the object of the correct type
    let object = glib::Object::with_type(gtype, &[])
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create {}: {}", type_name, e)))?;

    // Make sure it's actually a widget
    let widget = object.downcast::<gtk4::Widget>()
        .map_err(|_| mlua::Error::RuntimeError(format!("{} is not a widget type", type_name)))?;

    // Set properties if provided
    if let Some(props) = properties {
        for pair in props.pairs::<String, Value>() {
            let (key, value) = pair?;
            
            let gvalue = util::lua_value_to_gvalue(value)?;

            if let Err(e) = widget.set_property(&key, &gvalue) {
                eprintln!("Failed to set property {} on {}: {}", key, type_name, e);
            }
        }
    }

    Ok(GtkWidget::new(widget))
}

fn create_gtk_widget_table(lua: &Lua, app: &Application) -> LuaResult<Table> {
    let gtk_table = lua.create_table()?;

    // Dynamic widget creation with property support
    let widget_types = vec![
        "GtkApplicationWindow", "GtkWindow", "GtkButton", "GtkLabel", 
        "GtkEntry", "GtkBox", "GtkGrid", "GtkNotebook", "GtkPaned",
        "GtkScrolledWindow", "GtkViewport", "GtkFrame", "GtkAspectFrame",
        "GtkCheckButton", "GtkRadioButton", "GtkToggleButton", "GtkSwitch",
        "GtkScale", "GtkSpinButton", "GtkProgressBar", "GtkLevelBar",
        "GtkImage", "GtkPicture", "GtkVideo", "GtkSpinner",
        "GtkTextView", "GtkTreeView", "GtkListBox", "GtkFlowBox",
        "GtkMenuButton", "GtkDropDown", "GtkComboBox", "GtkSearchEntry",
        "GtkPasswordEntry", "GtkCalendar", "GtkColorButton", "GtkFontButton",
        "GtkFileChooserButton", "GtkSeparator", "GtkExpander"
    ];

    for widget_type in widget_types {
        let type_name = widget_type.to_string();
        let short_name = type_name.strip_prefix("Gtk").unwrap_or(&type_name);
        
        let constructor = lua.create_function(move |_, props: Option<Table>| {
            create_gtk_widget(type_name.clone(), props)
        })?;
        
        gtk_table.set(short_name, constructor)?;
    }

    // Special handling for ApplicationWindow to connect to the app
    let app_clone = app.clone();
    let create_app_window = lua.create_function(move |_, props: Option<Table>| {
        // Create ApplicationWindow with the actual app instance
        let window = gtk4::ApplicationWindow::new(&app_clone);
        
        // Set properties if provided
        if let Some(props) = props {
            for pair in props.pairs::<String, Value>() {
                let (key, value) = pair?;
                
                let gvalue = util::lua_value_to_gvalue(value)?;

                if let Err(e) = window.set_property(&key, &gvalue) {
                    eprintln!("Failed to set property {} on ApplicationWindow: {}", key, e);
                }
            }
        }

        Ok(GtkWidget::new(window.upcast()))
    })?;

    // Override ApplicationWindow constructor
    gtk_table.set("ApplicationWindow", create_app_window)?;

    Ok(gtk_table)
}

fn create_layer_shell_table(lua: &Lua) -> LuaResult<Table> {
    let layer_shell = lua.create_table()?;
    
    layer_shell.set("init", lua.create_function(|_, widget: GtkWidget| {
        if let Some(window) = widget.widget.downcast_ref::<gtk4::ApplicationWindow>() {
            window.init_layer_shell();
        } else if let Some(window) = widget.widget.downcast_ref::<gtk4::Window>() {
            window.init_layer_shell();
        }
        Ok(())
    })?)?;

    layer_shell.set("set_layer", lua.create_function(|_, (widget, layer): (GtkWidget, String)| {
        let layer_value = match layer.as_str() {
            "background" => Layer::Background,
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => Layer::Overlay,
        };

        if let Some(window) = widget.widget.downcast_ref::<gtk4::ApplicationWindow>() {
            window.set_layer(layer_value);
        } else if let Some(window) = widget.widget.downcast_ref::<gtk4::Window>() {
            window.set_layer(layer_value);
        }
        Ok(())
    })?)?;

    layer_shell.set("set_anchor", lua.create_function(|_, (widget, edges): (GtkWidget, Vec<String>)| {
        let set_anchors = |window: &dyn LayerShell| {
            // Clear all anchors first
            for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
                window.set_anchor(edge, false);
            }
           
            // Set specified anchors
            for edge_str in edges {
                let edge = match edge_str.as_str() {
                    "top" => Some(Edge::Top),
                    "bottom" => Some(Edge::Bottom),
                    "left" => Some(Edge::Left),
                    "right" => Some(Edge::Right),
                    _ => None,
                };
                if let Some(e) = edge {
                    window.set_anchor(e, true);
                }
            }
        };

        if let Some(window) = widget.widget.downcast_ref::<gtk4::ApplicationWindow>() {
            set_anchors(window);
        } else if let Some(window) = widget.widget.downcast_ref::<gtk4::Window>() {
            set_anchors(window);
        }
        Ok(())
    })?)?;

    Ok(layer_shell)
}

fn register_utility_functions(lua: &Lua, globals: &Table) -> LuaResult<()> {
    // Print function
    globals.set("print", lua.create_function(|_, msg: String| {
        println!("[Lua] {}", msg);
        Ok(())
    })?)?;

    Ok(())
}
