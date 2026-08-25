use serde_json::Value;

pub fn assert_no_generated_uri(value: &Value) {
    assert_no_generated_uri_at(value, value);
}

pub fn assert_no_generated_uri_or_zero_range(value: &Value) {
    assert_no_generated_uri(value);
    assert_no_zero_range_at(value, value);
}

fn assert_no_generated_uri_at(root: &Value, value: &Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                assert_no_generated_uri_at(root, value);
            }
        }
        Value::Object(object) => {
            for key in ["uri", "targetUri"] {
                if let Some(uri) = object.get(key).and_then(Value::as_str) {
                    assert!(
                        !uri_leaks_generated_projection(uri),
                        "response leaked generated URI {uri}: {root:#}"
                    );
                }
            }
            for value in object.values() {
                assert_no_generated_uri_at(root, value);
            }
        }
        _ => {}
    }
}

fn assert_no_zero_range_at(root: &Value, value: &Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                assert_no_zero_range_at(root, value);
            }
        }
        Value::Object(object) => {
            for key in ["range", "targetRange", "targetSelectionRange"] {
                if let Some(range) = object.get(key) {
                    assert!(
                        !range_starts_at_zero(range),
                        "response leaked generated zero range: {root:#}"
                    );
                }
            }
            for value in object.values() {
                assert_no_zero_range_at(root, value);
            }
        }
        _ => {}
    }
}

fn range_starts_at_zero(range: &Value) -> bool {
    range.get("start").is_some_and(position_is_zero)
}

fn position_is_zero(position: &Value) -> bool {
    position.get("line").and_then(Value::as_u64) == Some(0)
        && position.get("character").and_then(Value::as_u64) == Some(0)
}

fn uri_leaks_generated_projection(uri: &str) -> bool {
    uri.contains(".vue.ts")
}

#[cfg(test)]
mod tests {
    use super::uri_leaks_generated_projection;

    #[test]
    fn detects_ts_and_tsx_projection_uris() {
        assert!(uri_leaks_generated_projection(
            "file:///repo/src/App.vue.ts"
        ));
        assert!(uri_leaks_generated_projection(
            "file:///repo/src/App.vue.tsx"
        ));
        assert!(uri_leaks_generated_projection(
            "file:///repo/src/App.vue.ts?x=1"
        ));
        assert!(!uri_leaks_generated_projection("file:///repo/src/App.vue"));
    }
}
