
use gtk4::{prelude::*, Widget, glib};
use mlua::{AnyUserData, Function, UserData, UserDataMethods, Value};

use crate::lua::util;

#[derive(Debug)]
pub struct GtkWidget {
    pub widget: Widget,
}

impl GtkWidget {
    pub fn new(widget: Widget) -> Self {
        Self { widget }
    }
}

impl UserData for GtkWidget {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Set a property
        methods.add_method("set", |_, this, (property, value): (String, Value)| {
            let gvalue = util::lua_value_to_gvalue(value)?;
            this.widget.set_property(&property, &gvalue);
            Ok(())
        });

        // Get a property
        methods.add_method("get", |lua, this, property: String| {
            let gvalue: glib::Value = this.widget.property(&property);
            util::gvalue_to_lua_value(lua, gvalue)
        });

        // Show/hide widget
        methods.add_method("show", |_, this, ()| {
            this.widget.set_visible(true);
            Ok(())
        });

        methods.add_method("hide", |_, this, ()| {
            this.widget.set_visible(false);
            Ok(())
        });

        // Add a child widget
        methods.add_method("add", |_, this, child_ud: AnyUserData| {
            let child = child_ud.borrow::<GtkWidget>()?;
            util::add_child_to_container(&this.widget, &child.widget)
        });

        methods.add_method("connect", |_, this, (signal_name, callback): (String, Function)| {
            this.widget.connect_local(&signal_name, false, move |_| {
                if let Err(e) = callback.call::<()>(()) {
                    eprintln!("Signal callback error: {}", e);
                }
                None
            });
            Ok(())
        });



        methods.add_method("attach", |_, this, table: mlua::Table| {
            if let Some(grid) = this.widget.downcast_ref::<gtk4::Grid>() {
                // Get the child as AnyUserData first
                let child_ud: mlua::AnyUserData = table.get(1)?;
                // Borrow it as GtkWidget
                let child: mlua::UserDataRef<GtkWidget> = child_ud.borrow::<GtkWidget>()?;

                let left: i32 = table.get(2)?;
                let top: i32 = table.get(3)?;
                let width: i32 = table.get(4)?;
                let height: i32 = table.get(5)?;

                grid.attach(&child.widget, left, top, width, height);
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError("attach() only works on Grid widgets".to_string()))
            }
        });


        // Return the widget type name
        methods.add_method("type_name", |_, this, ()| {
            Ok(this.widget.type_().name().to_string())
        });
    }
}

