use crate::scripting::traits::ScriptValue;
use gtk4::glib::translate::*;
use gtk4::glib::{EnumClass, StrV, Type, Value};
use gtk4::prelude::*;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum ConversionError {
    UnsupportedType(String),
    InvalidValue(String),
    EnumConversion(String),
    Other(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnsupportedType(e) => {
                write!(f, "Unsupported type for conversion: {}", e)
            }
            ConversionError::InvalidValue(e) => write!(f, "Invalid value for conversion: {}", e),
            ConversionError::EnumConversion(e) => write!(f, "Enum conversion error: {}", e),
            ConversionError::Other(e) => write!(f, "Conversion error: {}", e),
        }
    }
}

impl StdError for ConversionError {}

impl From<ConversionError> for mlua::Error {
    fn from(err: ConversionError) -> Self {
        mlua::Error::external(err)
    }
}

pub struct GenericConverter;

impl GenericConverter {
    fn convert_string_to_gvalue<T: ScriptValue>(
        val: &T,
        target: Type,
    ) -> std::result::Result<Value, ConversionError> {
        let s = val
            .as_string()
            .ok_or_else(|| ConversionError::InvalidValue("Expected string".to_string()))?;
        if target == String::static_type() {
            Ok(s.to_value())
        } else if target.is_a(Type::ENUM) {
            if let Some(enum_class) = EnumClass::with_type(target) {
                if let Some(enum_val) = enum_class.value_by_nick(&s) {
                    unsafe {
                        let mut v = Value::from_type(target);
                        gobject_sys::g_value_set_enum(v.to_glib_none_mut().0, enum_val.value());
                        Ok(v)
                    }
                } else {
                    Err(ConversionError::EnumConversion(format!(
                        "Invalid enum value '{}' for type {:?}",
                        s, target
                    )))
                }
            } else {
                Err(ConversionError::UnsupportedType(format!(
                    "Target type {:?} is not an enum",
                    target
                )))
            }
        } else {
            Err(ConversionError::UnsupportedType(format!(
                "Cannot convert string to target type {:?}",
                target
            )))
        }
    }

    fn convert_bool_to_gvalue<T: ScriptValue>(
        val: &T,
        target: Type,
    ) -> std::result::Result<Value, ConversionError> {
        if target == bool::static_type() {
            val.as_bool()
                .map(|b| b.to_value())
                .ok_or_else(|| ConversionError::InvalidValue("Expected boolean".to_string()))
        } else {
            Err(ConversionError::UnsupportedType(format!(
                "Cannot convert boolean to target type {:?}",
                target
            )))
        }
    }

    fn convert_number_to_gvalue<T: ScriptValue>(
        val: &T,
        target: Type,
    ) -> std::result::Result<Value, ConversionError> {
        let n = val
            .as_number()
            .ok_or_else(|| ConversionError::InvalidValue("Expected number".to_string()))?;
        if target == i32::static_type() {
            Ok((n as i32).to_value())
        } else if target == u32::static_type() {
            Ok((n as u32).to_value())
        } else if target == i64::static_type() {
            Ok((n as i64).to_value())
        } else if target == f64::static_type() {
            Ok(n.to_value())
        } else if target == f32::static_type() {
            Ok((n as f32).to_value())
        } else {
            Err(ConversionError::UnsupportedType(format!(
                "Cannot convert number to target type {:?}",
                target
            )))
        }
    }

    fn convert_array_to_gvalue_strv<T: ScriptValue>(
        val: &T,
        target: Type,
    ) -> std::result::Result<Value, ConversionError> {
        if target == StrV::static_type() {
            let items = val
                .get_array_items()
                .ok_or_else(|| ConversionError::InvalidValue("Expected array".to_string()))?;
            let strings: Vec<String> = items.into_iter().filter_map(|v| v.as_string()).collect();
            Ok(strings.to_value())
        } else {
            Err(ConversionError::UnsupportedType(format!(
                "Cannot convert array to target type {:?}",
                target
            )))
        }
    }

    pub fn to_gvalue<T: ScriptValue>(
        val: &T,
        target: Type,
    ) -> std::result::Result<Value, ConversionError> {
        if val.is_string() {
            return Self::convert_string_to_gvalue(val, target);
        }
        if val.is_bool() {
            return Self::convert_bool_to_gvalue(val, target);
        }
        if val.is_number() {
            return Self::convert_number_to_gvalue(val, target);
        }
        if val.get_array_items().is_some() {
            return Self::convert_array_to_gvalue_strv(val, target);
        }
        Err(ConversionError::UnsupportedType(format!(
            "No suitable conversion found for value of type {:?} to target type {:?}",
            val, target
        )))
    }
}
