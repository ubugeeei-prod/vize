//! Advance archived compiler results across Vue binding-kind corrections.

use serde_json::Value;
use vize_carton::{String, cstr};

pub fn advance(mut expected: Value, vue_imports: &[&str]) -> Result<Value, String> {
    advance_at(&mut expected, "/Ok/bindings/bindings", vue_imports)?;
    Ok(expected)
}

pub fn advance_at(expected: &mut Value, pointer: &str, vue_imports: &[&str]) -> Result<(), String> {
    let bindings = expected
        .pointer_mut(pointer)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| cstr!("missing archived binding table at {pointer}"))?;
    for name in vue_imports {
        let previous = bindings
            .get(*name)
            .and_then(Value::as_str)
            .ok_or_else(|| cstr!("missing archived Vue import `{name}`"))?;
        if previous != "setup-maybe-ref" {
            return Err(cstr!(
                "archived Vue import `{name}` was `{previous}` before the correction"
            ));
        }
        bindings.insert((*name).to_owned(), Value::String("setup-const".to_owned()));
    }
    Ok(())
}
