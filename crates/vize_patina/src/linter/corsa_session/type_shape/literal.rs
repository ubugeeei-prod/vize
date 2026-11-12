//! Literal values of a constrained type, as the shape fact spells them.
use super::super::checker_memo::CheckerMemo;
use super::{BIGINT_LITERAL, BOOLEAN_LITERAL, NUMBER_LITERAL, STRING_LITERAL, ShapeCache, UNION};
use corsa::api::TypeResponse;
use serde_json::{Value, json};

/// Literal payload: strings and numbers as JSON, bigints as `{ "bigint": text }`
/// and boolean literals as JSON booleans.
pub(super) async fn literal_value(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<Option<Value>> {
    if ty.flags & BOOLEAN_LITERAL != 0 && ty.flags & UNION == 0 {
        if let Some(value) = cache.booleans.get(&ty.id) {
            return Ok(Some(json!(*value)));
        }
        let text = session
            .client()
            .type_to_string(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
                None,
                None,
            )
            .await
            .unwrap_or_default();
        let value = text.trim() == "true";
        cache.booleans.insert(ty.id.clone(), value);
        return Ok(Some(json!(value)));
    }
    if ty.flags & (STRING_LITERAL | NUMBER_LITERAL | BIGINT_LITERAL) == 0 {
        return Ok(None);
    }
    let Some(value) = ty.value.clone() else {
        return Ok(None);
    };
    if ty.flags & BIGINT_LITERAL != 0 {
        return Ok(Some(json!({ "bigint": bigint_text(&value) })));
    }
    Ok(Some(value))
}

pub(super) fn bigint_text(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let negative = map
                .get("negative")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let digits = map
                .get("base10Value")
                .and_then(Value::as_str)
                .unwrap_or("0");
            if negative && digits != "0" {
                Value::String(["-", digits].concat())
            } else {
                Value::String(digits.into())
            }
        }
        Value::String(text) => Value::String(text.trim_end_matches('n').into()),
        other => other.clone(),
    }
}
