use super::*;
use serde_json::Value;
use vize_l0::cstr;

macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition { return Err(cstr!($($message)+)); }
    };
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("missing {key}"))
}

mod hygiene;
pub(crate) mod original_input_model;
