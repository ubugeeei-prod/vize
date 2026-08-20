use serde_json::Value;

use super::ReportError;

/// Schema keys that annotate rather than validate.
const ANNOTATION_KEYWORDS: [&str; 4] = ["$schema", "$id", "title", "description"];
/// Validation keywords this subset implements.
const IMPLEMENTED_KEYWORDS: [&str; 7] = [
    "type",
    "required",
    "properties",
    "additionalProperties",
    "minimum",
    "minLength",
    "pattern",
];

pub(super) fn validate(schema: &Value, instance: &Value, path: &str) -> Result<(), ReportError> {
    let Some(schema_object) = schema.as_object() else {
        return Err(ReportError::SchemaNotAnObject { path: path.into() });
    };
    for keyword in schema_object.keys() {
        let known = ANNOTATION_KEYWORDS.contains(&keyword.as_str())
            || IMPLEMENTED_KEYWORDS.contains(&keyword.as_str());
        if !known {
            return Err(ReportError::SchemaUnimplementedKeyword {
                path: path.into(),
                keyword: keyword.as_str().into(),
            });
        }
    }

    if let Some(expected) = schema_object.get("type") {
        check_type(expected, instance, path)?;
    }
    check_object_keywords(schema, instance, path)?;
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

fn check_type(expected: &Value, instance: &Value, path: &str) -> Result<(), ReportError> {
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
        Err(ReportError::SchemaType {
            path: path.into(),
            expected: expected_text.into(),
            found: json_type_name(instance).into(),
        })
    }
}

fn child_path(parent: &str, key: &str) -> Box<str> {
    let mut path = parent.to_owned();
    path.push('.');
    path.push_str(key);
    path.into()
}

fn check_object_keywords(schema: &Value, instance: &Value, path: &str) -> Result<(), ReportError> {
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
                return Err(ReportError::SchemaRequired {
                    path: path.into(),
                    property: property.into(),
                });
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
                        return Err(ReportError::SchemaUnexpectedProperty {
                            path: path.into(),
                            property: key.as_str().into(),
                        });
                    }
                }
            }
            _ => {
                return Err(ReportError::SchemaUnimplementedKeyword {
                    path: path.into(),
                    keyword: "additionalProperties (non-boolean form)".into(),
                });
            }
        }
    }

    if let Some(entries) = properties {
        for (key, subschema) in entries {
            if let Some(child) = instance_object.get(key.as_str()) {
                validate(subschema, child, &child_path(path, key))?;
            }
        }
    }
    Ok(())
}

fn check_scalar_bounds(schema: &Value, instance: &Value, path: &str) -> Result<(), ReportError> {
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_u64)
        && let Some(number) = instance.as_number()
    {
        let below = match number.as_u64() {
            Some(value) => value < minimum,
            None => number.as_f64().is_some_and(|value| value < minimum as f64),
        };
        if below {
            return Err(ReportError::SchemaMinimum {
                path: path.into(),
                minimum,
            });
        }
    }

    if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
        && let Some(text) = instance.as_str()
        && (text.chars().count() as u64) < min_length
    {
        return Err(ReportError::SchemaMinLength {
            path: path.into(),
            min_length,
        });
    }

    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
        && let Some(text) = instance.as_str()
    {
        let regex = regex_lite::Regex::new(pattern).map_err(|_| ReportError::SchemaBadPattern {
            path: path.into(),
            pattern: pattern.into(),
        })?;
        if !regex.is_match(text) {
            return Err(ReportError::SchemaPattern {
                path: path.into(),
                pattern: pattern.into(),
            });
        }
    }
    Ok(())
}
