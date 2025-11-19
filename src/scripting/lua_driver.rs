use crate::scripting::traits::ScriptValue;
use mlua::Value;

#[derive(Clone, Debug)]
pub struct LuaWrapper(pub Value);

impl ScriptValue for LuaWrapper {
    fn is_string(&self) -> bool {
        self.0.is_string()
    }
    fn is_number(&self) -> bool {
        self.0.is_number() || self.0.is_integer()
    }
    fn is_bool(&self) -> bool {
        self.0.is_boolean()
    }
    fn is_function(&self) -> bool {
        matches!(self.0, Value::Function(_))
    }

    fn as_string(&self) -> Option<String> {
        match &self.0 {
            Value::String(s) => s.to_str().ok().map(|x| x.to_string()),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match &self.0 {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i64> {
        match &self.0 {
            Value::Integer(i) => Some(*i),
            Value::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.0 {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn get_property(&self, key: &str) -> Option<Self> {
        if let Value::Table(t) = &self.0 {
            t.get::<Value>(key).ok().map(LuaWrapper)
        } else {
            None
        }
    }

    fn get_array_items(&self) -> Option<Vec<Self>> {
        if let Value::Table(t) = &self.0 {
            let items: Result<Vec<_>, _> = t.sequence_values().map(|v| v.map(LuaWrapper)).collect();
            items.ok()
        } else {
            None
        }
    }

    fn get_map_entries(&self) -> Option<Vec<(String, Self)>> {
        if let Value::Table(t) = &self.0 {
            let entries: Result<Vec<_>, _> = t
                .pairs::<String, Value>()
                .map(|r| r.map(|(k, v)| (k, LuaWrapper(v))))
                .collect();
            entries.ok()
        } else {
            None
        }
    }

    fn call_void(&self) -> Result<(), String> {
        if let Value::Function(f) = &self.0 {
            f.call::<()>(()).map_err(|e| e.to_string())
        } else {
            Err("Not a function".into())
        }
    }
}
