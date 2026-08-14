//! Minimal, strict JSON Schema subset validator.
//!
//! Implements exactly the keywords the committed schema uses (`type`,
//! `required`, `properties`, `additionalProperties: false`, `items`,
//! `minimum`, `minLength`, `pattern`, `const`) and rejects any other
//! validation keyword instead of skipping it, so a schema edit cannot
//! silently outrun the validator. Mirrors
//! `benchmarks/davinci_harness/src/report.rs`.

use serde_json::Value;

/// Schema keys that annotate rather than validate.
const ANNOTATION_KEYWORDS: [&str; 4] = ["$schema", "$id", "title", "description"];
/// Validation keywords this subset implements.
const IMPLEMENTED_KEYWORDS: [&str; 9] = [
    "type",
    "required",
    "properties",
    "additionalProperties",
    "items",
    "minimum",
    "minLength",
    "pattern",
    "const",
];

pub(super) fn validate(schema: &Value, instance: &Value, path: &str) -> Result<(), String> {
    let Some(schema_object) = schema.as_object() else {
        return Err(format!("schema at `{path}` must be a JSON object"));
    };
    for keyword in schema_object.keys() {
        let known = ANNOTATION_KEYWORDS.contains(&keyword.as_str())
            || IMPLEMENTED_KEYWORDS.contains(&keyword.as_str());
        if !known {
            return Err(format!(
                "schema keyword `{keyword}` at `{path}` is not implemented by this validator"
            ));
        }
    }

    if let Some(expected) = schema_object.get("type") {
        check_type(expected, instance, path)?;
    }
    if let Some(expected) = schema_object.get("const")
        && instance != expected
    {
        return Err(format!(
            "schema violation at `{path}`: value does not equal const `{expected}`"
        ));
    }
    check_object_keywords(schema, instance, path)?;
    check_array_items(schema, instance, path)?;
    check_scalar_bounds(schema, instance, path)?;
    Ok(())
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) => {
            if number.is_i64() || number.is_u64() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn type_entry_matches(entry: &Value, instance: &Value) -> bool {
    let Some(name) = entry.as_str() else {
        return false;
    };
    let actual = json_type_name(instance);
    // JSON Schema treats integers as a subset of numbers.
    name == actual || (name == "number" && actual == "integer")
}

fn check_type(expected: &Value, instance: &Value, path: &str) -> Result<(), String> {
    let matches = match expected {
        Value::Array(entries) => entries
            .iter()
            .any(|entry| type_entry_matches(entry, instance)),
        single => type_entry_matches(single, instance),
    };
    if matches {
        Ok(())
    } else {
        let mut expected_text = serde_json::to_string(expected).unwrap_or_default();
        expected_text.retain(|character| character != '"');
        Err(format!(
            "schema violation at `{path}`: expected {expected_text}, found {}",
            json_type_name(instance)
        ))
    }
}

fn check_object_keywords(schema: &Value, instance: &Value, path: &str) -> Result<(), String> {
    let Some(instance_object) = instance.as_object() else {
        // Non-object instances have nothing for the object keywords to
        // check; `type` has already policed whether that is legal.
        return Ok(());
    };

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for entry in required {
            if let Some(property) = entry.as_str()
                && !instance_object.contains_key(property)
            {
                return Err(format!(
                    "schema violation at `{path}`: missing required property `{property}`"
                ));
            }
        }
    }

    let properties = schema.get("properties").and_then(Value::as_object);

    if let Some(additional) = schema.get("additionalProperties") {
        match additional {
            Value::Bool(true) => {}
            Value::Bool(false) => {
                for key in instance_object.keys() {
                    let declared =
                        properties.is_some_and(|entries| entries.contains_key(key.as_str()));
                    if !declared {
                        return Err(format!(
                            "schema violation at `{path}`: unexpected property `{key}`"
                        ));
                    }
                }
            }
            _ => {
                return Err(format!(
                    "schema keyword `additionalProperties (non-boolean form)` at `{path}` is not implemented by this validator"
                ));
            }
        }
    }

    if let Some(entries) = properties {
        for (key, subschema) in entries {
            if let Some(child) = instance_object.get(key.as_str()) {
                validate(subschema, child, &format!("{path}.{key}"))?;
            }
        }
    }
    Ok(())
}

fn check_array_items(schema: &Value, instance: &Value, path: &str) -> Result<(), String> {
    let Some(items) = schema.get("items") else {
        return Ok(());
    };
    let Some(entries) = instance.as_array() else {
        // `type` has already policed whether a non-array is legal here.
        return Ok(());
    };
    for (index, entry) in entries.iter().enumerate() {
        validate(items, entry, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

fn check_scalar_bounds(schema: &Value, instance: &Value, path: &str) -> Result<(), String> {
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_u64)
        && let Some(number) = instance.as_number()
    {
        let below = match number.as_u64() {
            Some(value) => value < minimum,
            None => number.as_f64().is_some_and(|value| value < minimum as f64),
        };
        if below {
            return Err(format!(
                "schema violation at `{path}`: value is below minimum {minimum}"
            ));
        }
    }

    if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
        && let Some(text) = instance.as_str()
        && (text.chars().count() as u64) < min_length
    {
        return Err(format!(
            "schema violation at `{path}`: string is shorter than minLength {min_length}"
        ));
    }

    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
        && let Some(text) = instance.as_str()
    {
        let regex = regex_lite::Regex::new(pattern)
            .map_err(|_| format!("schema pattern `{pattern}` at `{path}` does not compile"))?;
        if !regex.is_match(text) {
            return Err(format!(
                "schema violation at `{path}`: string does not match pattern `{pattern}`"
            ));
        }
    }
    Ok(())
}
