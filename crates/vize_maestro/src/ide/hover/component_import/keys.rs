//! Property names in displayed TypeScript contracts.

pub(super) fn field_name(name: &str) -> String {
    if name.is_ascii() && vize_l0::is_simple_identifier(name) {
        name.to_string()
    } else {
        quoted_name(name)
    }
}

pub(super) fn quoted_name(name: &str) -> String {
    serde_json::Value::String(name.to_string()).to_string()
}
