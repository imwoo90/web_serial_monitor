use std::fmt::Display;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum LogError {
    Js(JsValue),
    Storage(String),
    Encoding(String),
    Regex(String),
}

impl Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::Js(v) => write!(f, "JS Error: {:?}", v),
            LogError::Storage(s) => write!(f, "Storage Error: {}", s),
            LogError::Encoding(s) => write!(f, "Encoding Error: {}", s),
            LogError::Regex(s) => write!(f, "Regex Error: {}", s),
        }
    }
}

impl From<JsValue> for LogError {
    fn from(v: JsValue) -> Self {
        LogError::Js(v)
    }
}

impl From<LogError> for JsValue {
    fn from(e: LogError) -> Self {
        JsValue::from_str(&e.to_string())
    }
}
