use crate::scripting::traits::ScriptValue;
use gtk4::glib::{self, Object};
use gtk4::prelude::*;

pub struct SignalConnector;

impl SignalConnector {
    pub fn connect<T: ScriptValue + 'static>(widget: &Object, name: &str, func: T) {
        let signal_name = name.to_string();

        widget.connect_local(name, false, move |_values: &[glib::Value]| {
            if let Err(e) = func.call_void() {
                eprintln!("Signal Error [{}]: {}", signal_name, e);
            }
            None
        });
    }
}
