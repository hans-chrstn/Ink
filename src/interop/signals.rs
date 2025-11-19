use crate::scripting::traits::{ScriptArg, ScriptValue};
use gtk4::glib::{self, Object};
use gtk4::prelude::*;

pub struct SignalConnector;

impl SignalConnector {
    pub fn connect<T: ScriptValue + 'static>(widget: &Object, name: &str, func: T) {
        let signal_name = name.replace("_", "-");

        widget.connect_local(name, false, move |values: &[glib::Value]| {
            let args: Vec<ScriptArg> = values
                .iter()
                .map(|val| {
                    if let Ok(w) = val.get::<gtk4::Widget>() {
                        ScriptArg::Widget(w)
                    } else if let Ok(s) = val.get::<String>() {
                        ScriptArg::String(s)
                    } else if let Ok(b) = val.get::<bool>() {
                        ScriptArg::Bool(b)
                    } else if let Ok(n) = val.get::<f64>() {
                        ScriptArg::Number(n)
                    } else if let Ok(i) = val.get::<i64>() {
                        ScriptArg::Number(i as f64)
                    } else {
                        ScriptArg::Nil
                    }
                })
                .collect();

            let handle_return = |ret: ScriptArg| -> Option<glib::Value> {
                match ret {
                    ScriptArg::Bool(b) => Some(b.to_value()),
                    ScriptArg::Number(n) => Some(n.to_value()),
                    ScriptArg::String(s) => Some(s.to_value()),
                    ScriptArg::Nil => {
                        if signal_name == "state-set" {
                            Some(true.to_value())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            match func.call(args) {
                Ok(ret) => handle_return(ret),
                Err(e) => {
                    eprintln!("Signal Error [{}]: {}", signal_name, e);
                    if signal_name == "state-set" {
                        Some(true.to_value())
                    } else {
                        None
                    }
                }
            }
        });
    }
}
