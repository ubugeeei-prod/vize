//! Minimal, strict JSON Schema subset validator.
//!
//! Implements exactly the keywords used by the committed Davinci schemas and
//! rejects any unsupported validation keyword instead of silently skipping it.

use compact_str::{CompactString, format_compact};
use serde_json::Value;
use thiserror::Error;

/// A precise validation or validator-capability failure.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    /// A schema node was not a JSON object.
    #[error("schema at `{path}` must be a JSON object")]
    NotObject { path: CompactString },
    /// The schema uses a keyword outside the deliberately small subset.
    #[error("schema keyword `{keyword}` at `{path}` is not implemented by this validator")]
    UnimplementedKeyword {
        path: CompactString,
        keyword: CompactString,
    },
    /// An instance has the wrong JSON type.
    #[error("schema violation at `{path}`: expected {expected}, found {found}")]
    Type {
        path: CompactString,
        expected: CompactString,
        found: &'static str,
    },
    /// An instance differs from a schema `const`.
    #[error("schema violation at `{path}`: value does not equal const `{expected}`")]
    Const {
        path: CompactString,
        expected: CompactString,
    },
    /// A required object property is absent.
    #[error("schema violation at `{path}`: missing required property `{property}`")]
    Required {
        path: CompactString,
        property: CompactString,
    },
    /// An object contains a property forbidden by its schema.
    #[error("schema violation at `{path}`: unexpected property `{property}`")]
    UnexpectedProperty {
        path: CompactString,
        property: CompactString,
    },
    /// A numeric instance is below its minimum.
    #[error("schema violation at `{path}`: value is below minimum {minimum}")]
    Minimum { path: CompactString, minimum: u64 },
    /// A string instance is shorter than its minimum length.
    #[error("schema violation at `{path}`: string is shorter than minLength {min_length}")]
    MinLength {
        path: CompactString,
        min_length: u64,
    },
    /// A string instance does not match its schema pattern.
    #[error("schema violation at `{path}`: string does not match pattern `{pattern}`")]
    Pattern {
        path: CompactString,
        pattern: CompactString,
    },
    /// A schema pattern itself is invalid.
    #[error("schema pattern `{pattern}` at `{path}` does not compile")]
    BadPattern {
        path: CompactString,
        pattern: CompactString,
    },
}

impl SchemaError {
    /// Render the stable diagnostic text without allocating a standard string.
    #[must_use]
    pub fn message(&self) -> CompactString {
        format_compact!("{self}")
    }
}

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

/// Validate `instance` against the strict Davinci schema subset.
pub fn validate(schema: &Value, instance: &Value, path: &str) -> Result<(), SchemaError> {
    let Some(schema_object) = schema.as_object() else {
        return Err(SchemaError::NotObject { path: path.into() });
    };
    for keyword in schema_object.keys() {
        let known = ANNOTATION_KEYWORDS.contains(&keyword.as_str())
            || IMPLEMENTED_KEYWORDS.contains(&keyword.as_str());
        if !known {
            return Err(SchemaError::UnimplementedKeyword {
                path: path.into(),
                keyword: keyword.as_str().into(),
            });
        }
    }

    if let Some(expected) = schema_object.get("type") {
        check_type(expected, instance, path)?;
    }
    if let Some(expected) = schema_object.get("const")
        && instance != expected
    {
        return Err(SchemaError::Const {
            path: path.into(),
            expected: serde_json::to_string(expected).unwrap_or_default().into(),
        });
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
    name == actual || (name == "number" && actual == "integer")
}

fn check_type(expected: &Value, instance: &Value, path: &str) -> Result<(), SchemaError> {
    let matches = match expected {
        Value::Array(entries) => entries
            .iter()
            .any(|entry| type_entry_matches(entry, instance)),
        single => type_entry_matches(single, instance),
    };
    if matches {
        return Ok(());
    }

    let mut expected_text = serde_json::to_string(expected).unwrap_or_default();
    expected_text.retain(|character| character != '"');
    Err(SchemaError::Type {
        path: path.into(),
        expected: expected_text.into(),
        found: json_type_name(instance),
    })
}

fn check_object_keywords(schema: &Value, instance: &Value, path: &str) -> Result<(), SchemaError> {
    let Some(instance_object) = instance.as_object() else {
        return Ok(());
    };

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for entry in required {
            if let Some(property) = entry.as_str()
                && !instance_object.contains_key(property)
            {
                return Err(SchemaError::Required {
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
                        return Err(SchemaError::UnexpectedProperty {
                            path: path.into(),
                            property: key.as_str().into(),
                        });
                    }
                }
            }
            _ => {
                return Err(SchemaError::UnimplementedKeyword {
                    path: path.into(),
                    keyword: "additionalProperties (non-boolean form)".into(),
                });
            }
        }
    }

    if let Some(entries) = properties {
        for (key, subschema) in entries {
            if let Some(child) = instance_object.get(key.as_str()) {
                validate(subschema, child, &format_compact!("{path}.{key}"))?;
            }
        }
    }
    Ok(())
}

fn check_array_items(schema: &Value, instance: &Value, path: &str) -> Result<(), SchemaError> {
    let Some(items) = schema.get("items") else {
        return Ok(());
    };
    let Some(entries) = instance.as_array() else {
        return Ok(());
    };
    for (index, entry) in entries.iter().enumerate() {
        validate(items, entry, &format_compact!("{path}[{index}]"))?;
    }
    Ok(())
}

fn check_scalar_bounds(schema: &Value, instance: &Value, path: &str) -> Result<(), SchemaError> {
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_u64)
        && let Some(number) = instance.as_number()
    {
        let below = match number.as_u64() {
            Some(value) => value < minimum,
            None => number.as_f64().is_some_and(|value| value < minimum as f64),
        };
        if below {
            return Err(SchemaError::Minimum {
                path: path.into(),
                minimum,
            });
        }
    }

    if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
        && let Some(text) = instance.as_str()
        && (text.chars().count() as u64) < min_length
    {
        return Err(SchemaError::MinLength {
            path: path.into(),
            min_length,
        });
    }

    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
        && let Some(text) = instance.as_str()
    {
        let regex = regex_lite::Regex::new(pattern).map_err(|_| SchemaError::BadPattern {
            path: path.into(),
            pattern: pattern.into(),
        })?;
        if !regex.is_match(text) {
            return Err(SchemaError::Pattern {
                path: path.into(),
                pattern: pattern.into(),
            });
        }
    }
    Ok(())
}
