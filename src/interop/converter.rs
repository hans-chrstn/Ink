use crate::scripting::traits::ScriptValue;
use gtk4::glib::translate::*;
use gtk4::glib::{EnumClass, Type, Value};
use gtk4::prelude::*;

pub struct GenericConverter;

impl GenericConverter {
    pub fn to_gvalue<T: ScriptValue>(val: &T, target: Type) -> Option<Value> {
        if val.is_string() {
            let s = val.as_string()?;

            if target == String::static_type() {
                return Some(s.to_value());
            }

            if target.is_a(Type::ENUM) {
                if let Some(enum_class) = EnumClass::with_type(target) {
                    if let Some(enum_val) = enum_class.value_by_nick(&s) {
                        unsafe {
                            let mut v = Value::from_type(target);

                            gobject_sys::g_value_set_enum(v.to_glib_none_mut().0, enum_val.value());

                            return Some(v);
                        }
                    }
                }
            }
        }

        if val.is_bool() && target == bool::static_type() {
            return val.as_bool().map(|b| b.to_value());
        }

        if val.is_number() {
            let n = val.as_number()?;
            if target == i32::static_type() {
                return Some((n as i32).to_value());
            } else if target == u32::static_type() {
                return Some((n as u32).to_value());
            } else if target == i64::static_type() {
                return Some((n as i64).to_value());
            } else if target == f64::static_type() {
                return Some(n.to_value());
            } else if target == f32::static_type() {
                return Some((n as f32).to_value());
            }
        }

        None
    }
}
