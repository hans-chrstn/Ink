use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ScriptArg {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Widget(gtk4::Widget),
}

pub trait ScriptValue: Sized + Clone + Debug {
    fn is_string(&self) -> bool;
    fn is_number(&self) -> bool;
    fn is_bool(&self) -> bool;
    fn is_function(&self) -> bool;

    fn as_string(&self) -> Option<String>;
    fn as_number(&self) -> Option<f64>;
    fn as_integer(&self) -> Option<i64>;
    fn as_bool(&self) -> Option<bool>;

    fn get_property(&self, key: &str) -> Option<Self>;
    fn get_array_items(&self) -> Option<Vec<Self>>;
    fn get_map_entries(&self) -> Option<Vec<(String, Self)>>;

    fn call(&self, args: Vec<ScriptArg>) -> Result<ScriptArg, String>;
}
