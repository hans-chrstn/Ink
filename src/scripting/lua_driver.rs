use crate::scripting::traits::{ScriptArg, ScriptValue};
use crate::scripting::widget_wrapper::LuaWidget;
use mlua::{IntoLua, IntoLuaMulti, MultiValue, Value};
#[derive(Clone, Debug)]
pub struct LuaWrapper(pub Value);
impl IntoLua for ScriptArg {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
        match self {
            ScriptArg::Nil => Ok(Value::Nil),
            ScriptArg::Bool(b) => b.into_lua(lua),
            ScriptArg::Number(n) => n.into_lua(lua),
            ScriptArg::String(s) => s.into_lua(lua),
            ScriptArg::Widget(w) => LuaWidget(w).into_lua(lua),
        }
    }
}
struct ScriptArgs(Vec<ScriptArg>);
impl IntoLuaMulti for ScriptArgs {
    fn into_lua_multi(self, lua: &mlua::Lua) -> mlua::Result<MultiValue> {
        let mut vals = Vec::new();
        for arg in self.0 {
            vals.push(arg.into_lua(lua)?);
        }
        Ok(MultiValue::from_vec(vals))
    }
}
fn lua_to_script_arg(val: Value) -> ScriptArg {
    match val {
        Value::Boolean(b) => ScriptArg::Bool(b),
        Value::String(s) => {
            ScriptArg::String(s.to_str().ok().map(|bs| bs.to_string()).unwrap_or_default())
        }
        Value::Number(n) => ScriptArg::Number(n),
        Value::Integer(i) => ScriptArg::Number(i as f64),
        _ => ScriptArg::Nil,
    }
}
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
            t.sequence_values()
                .map(|v| v.map(LuaWrapper))
                .collect::<Result<Vec<_>, _>>()
                .ok()
        } else {
            None
        }
    }
    fn get_map_entries(&self) -> Option<Vec<(String, Self)>> {
        if let Value::Table(t) = &self.0 {
            t.pairs::<String, Value>()
                .map(|r| r.map(|(k, v)| (k, LuaWrapper(v))))
                .collect::<Result<Vec<_>, _>>()
                .ok()
        } else {
            None
        }
    }
    fn call(&self, args: Vec<ScriptArg>) -> Result<ScriptArg, String> {
        if let Value::Function(f) = &self.0 {
            let result: Value = f.call(ScriptArgs(args)).map_err(|e| e.to_string())?;
            Ok(lua_to_script_arg(result))
        } else {
            Err("Not a function".into())
        }
    }
}
