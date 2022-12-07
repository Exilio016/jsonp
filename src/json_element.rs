use std::collections::HashMap;

pub enum JsonElement {
    Object(HashMap<String, JsonElement>),
    Array(Vec<JsonElement>),
    Str(String),
    Number(f64),
    Boolean(bool),
    Null,
}
