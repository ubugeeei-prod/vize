//! Configuration contracts checked against the packaged VS Code manifest.

use serde_json::Value;

use super::archive as editor_archive;

pub fn assert_configuration_defaults(package_json: &Value) -> Result<(), String> {
    let properties = package_json
        .pointer("/contributes/configuration/properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing contributes.configuration.properties".to_string())?;
    if properties
        .get("vize.enable")
        .and_then(|value| value.get("default"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("vize.enable has an unexpected default".to_string());
    }
    editor_archive::expect_json_string(
        properties
            .get("vize.serverPath")
            .ok_or("missing vize.serverPath")?,
        &["default"],
        "",
    )?;
    editor_archive::expect_json_string(
        properties
            .get("vize.trace.server")
            .ok_or("missing vize.trace.server")?,
        &["default"],
        "off",
    )?;
    let timeout_key = "vize.typecheck.timeoutMs";
    let timeout = properties
        .get(timeout_key)
        .ok_or_else(|| format!("missing {timeout_key}"))?;
    if timeout.get("type").and_then(Value::as_str) != Some("integer") {
        return Err(format!("{timeout_key} must have integer type"));
    }
    for (field, expected) in [
        ("default", 10_000),
        ("minimum", 1),
        ("maximum", u64::from(u32::MAX)),
    ] {
        if timeout.get(field).and_then(Value::as_u64) != Some(expected) {
            return Err(format!("{timeout_key}.{field} must be integer {expected}"));
        }
    }
    for (key, property) in properties {
        if key == timeout_key
            || key == "vize.serverPath"
            || key == "vize.trace.server"
            || (key.starts_with("vize.autoInsert.") && key != "vize.autoInsert.enable")
        {
            continue;
        }
        let expected = !matches!(
            key.as_str(),
            "vize.diagnostics.enable"
                | "vize.formatting.enable"
                | "vize.optionsApi.enable"
                | "vize.legacyVue2.enable"
                | "vize.autoInsert.enable"
        );
        if property.get("default").and_then(Value::as_bool) != Some(expected) {
            return Err(format!("{key} has an unexpected default"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::assert_configuration_defaults;
    use serde_json::{Map, Value, json};

    fn manifest() -> Value {
        serde_json::from_str(include_str!("../../../editors/vscode/package.json"))
            .expect("the actual extension manifest must parse")
    }

    fn properties(manifest: &mut Value) -> &mut Map<String, Value> {
        manifest
            .pointer_mut("/contributes/configuration/properties")
            .and_then(Value::as_object_mut)
            .expect("the actual extension manifest must declare configuration")
    }

    fn replace_field(package: &mut Value, key: &str, field: &str, value: Value) {
        properties(package)
            .get_mut(key)
            .and_then(Value::as_object_mut)
            .expect("the actual setting must be a schema object")
            .insert(field.to_string(), value);
    }

    #[test]
    fn current_packaged_manifest_accepts_the_integer_timeout_contract() {
        assert_configuration_defaults(&manifest()).expect("current defaults must pass");
    }

    #[test]
    fn missing_timeout_or_schema_fields_are_rejected() {
        let mut package = manifest();
        properties(&mut package).remove("vize.typecheck.timeoutMs");
        assert!(assert_configuration_defaults(&package).is_err());
        for field in ["type", "default", "minimum", "maximum"] {
            let mut package = manifest();
            properties(&mut package)
                .get_mut("vize.typecheck.timeoutMs")
                .and_then(Value::as_object_mut)
                .expect("timeout must be a schema object")
                .remove(field);
            assert!(assert_configuration_defaults(&package).is_err(), "{field}");
        }
    }

    #[test]
    fn changed_defaults_and_non_integer_values_are_rejected() {
        for value in [
            json!(0),
            json!(-1),
            json!(1),
            json!(5_000),
            json!(180_000),
            json!(10_000.0),
            json!("10000"),
            json!(true),
            json!(false),
            json!(null),
            json!([]),
            json!({}),
        ] {
            let mut package = manifest();
            replace_field(
                &mut package,
                "vize.typecheck.timeoutMs",
                "default",
                value.clone(),
            );
            assert!(assert_configuration_defaults(&package).is_err(), "{value}");
        }
        for value in [
            json!("number"),
            json!("boolean"),
            json!("string"),
            json!(null),
        ] {
            let mut package = manifest();
            replace_field(
                &mut package,
                "vize.typecheck.timeoutMs",
                "type",
                value.clone(),
            );
            assert!(assert_configuration_defaults(&package).is_err(), "{value}");
        }
    }

    #[test]
    fn timeout_bound_drift_is_rejected() {
        for (field, values) in [
            (
                "minimum",
                vec![json!(0), json!(-1), json!(2), json!(1.0), json!("1")],
            ),
            (
                "maximum",
                vec![
                    json!(4_294_967_294_u64),
                    json!(4_294_967_296_u64),
                    json!("4294967295"),
                    json!(true),
                ],
            ),
        ] {
            for value in values {
                let mut package = manifest();
                replace_field(
                    &mut package,
                    "vize.typecheck.timeoutMs",
                    field,
                    value.clone(),
                );
                assert!(
                    assert_configuration_defaults(&package).is_err(),
                    "{field}={value}"
                );
            }
        }
    }

    #[test]
    fn existing_boolean_and_string_default_guards_are_preserved() {
        let mut current = manifest();
        let boolean_keys = properties(&mut current)
            .iter()
            .filter(|(key, value)| {
                (key.as_str() == "vize.enable"
                    || key.ends_with(".enable")
                    || key.as_str() == "vize.server.preferWorkspace")
                    && value.get("default").and_then(Value::as_bool).is_some()
            })
            .map(|(key, value)| {
                (
                    key.clone(),
                    value
                        .get("default")
                        .and_then(Value::as_bool)
                        .expect("filtered boolean"),
                )
            })
            .collect::<Vec<_>>();
        for (key, original) in boolean_keys {
            for invalid in [json!(!original), json!(10_000), json!("true"), json!(null)] {
                let mut package = manifest();
                replace_field(&mut package, &key, "default", invalid.clone());
                assert!(
                    assert_configuration_defaults(&package).is_err(),
                    "{key}={invalid}"
                );
            }
        }
        for (key, invalid) in [
            ("vize.serverPath", json!(true)),
            ("vize.trace.server", json!(false)),
        ] {
            let mut package = manifest();
            replace_field(&mut package, key, "default", invalid);
            assert!(assert_configuration_defaults(&package).is_err(), "{key}");
        }
    }

    #[test]
    fn an_unknown_numeric_property_still_fails_closed() {
        let mut package = manifest();
        properties(&mut package).insert(
            "vize.futureNumeric".to_string(),
            json!({"type":"integer","default":10_000}),
        );
        assert!(assert_configuration_defaults(&package).is_err());
    }
}
