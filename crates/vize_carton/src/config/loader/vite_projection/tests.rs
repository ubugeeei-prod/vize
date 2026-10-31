use super::{check, project, read_embedded_schema, schema};
use serde_json::{Value, json};

#[test]
fn embedded_schema_decodes_backslash_pairs_before_json_escapes() {
    let runtime = r#"const vueOptionSchema = JSON.parse(
        '{"type":"string","pattern":"^\\\\s+$","title":"雪🌸","line":"\\n"}',
    );"#;
    assert_eq!(
        read_embedded_schema(runtime),
        Some(json!({
            "type": "string", "pattern": r"^\s+$", "title": "雪🌸", "line": "\n",
        }))
    );
    assert_eq!(
        read_embedded_schema(r#"JSON.parse('{"type":"object"}');"#),
        Some(json!({ "type": "object" }))
    );
}

#[test]
fn unsupported_js_escapes_and_ambiguous_literals_decline() {
    for runtime in [
        r#"JSON.parse('{"value":"\n"}');"#,
        r#"JSON.parse('{"value":"\u0061"}');"#,
        r#"JSON.parse('{"value":"\x61"}');"#,
        r#"JSON.parse('{"value":"it\'s"}');"#,
        r#"JSON.parse('{"value":"raw'quote"}');"#,
        r#"JSON.parse('{"value":"trailing\');"#,
        r#"JSON.parse('{}' + other);"#,
        r#"JSON.parse('{}', repair);"#,
        r#"JSON.parse('{}').repair();"#,
        r#"JSON.parse('{}')[0];"#,
        r#"JSON.parse('{}'), other;"#,
        r#"JSON.parse('{}')"#,
        r#"JSON.parse("{}");"#,
        r#"JSON.parse('{not-json}');"#,
        "JSON.parse('{\n\"type\":\"object\"}');",
        "JSON.parse('{\"value\":\"line\\\nbreak\"}');",
    ] {
        assert!(read_embedded_schema(runtime).is_none(), "{runtime}");
    }
}

#[test]
fn generated_schema_and_original_literal_sections_remain_readable() {
    let schema =
        schema().expect("the actual generated schema must not disable all literal configs");
    assert_eq!(
        schema["definitions"]["LintRuleOptions"]["properties"]["vue/v-on-event-hyphenation"]["oneOf"]
            [1]["properties"]["ignore"]["items"]["pattern"],
        json!(r"^(?!\s*$)(?![\s\S]*:exit$)")
    );
    let source = json!({
        "vue": {
            "compiler": { "sourceMap": false },
            "formatter": { "semi": false },
            "linter": { "ruleOptions": { "vue/v-on-event-hyphenation": "never" } },
        },
    });
    let before = source.clone();
    assert_eq!(
        project(source.as_object().unwrap(), false, "/authored"),
        Some(json!({
            "typeChecker": { "jsxTypecheck": true },
            "compiler": { "sourceMap": false },
            "formatter": { "semi": false },
            "linter": { "ruleOptions": { "vue/v-on-event-hyphenation": "never" } },
            "__vizeProjectRoot": "/authored",
        }))
    );
    assert_eq!(source, before);
}

#[test]
fn pattern_validation_is_never_silently_accepted_by_the_native_subset() {
    let rule = json!({ "type": "string", "pattern": "^allowed$" });
    for value in [json!("allowed"), json!("refused"), json!("")] {
        assert_eq!(check(&value, &rule, &rule), None, "{value}");
    }
    assert_eq!(check(&json!(0), &rule, &rule), Some(false));
    let empty_pattern = json!({ "type": "string", "pattern": "" });
    assert_eq!(
        check(&json!(""), &empty_pattern, &empty_pattern),
        Some(true)
    );
    let unknown_pattern = json!({ "pattern": true });
    assert_eq!(
        check(&json!("text"), &unknown_pattern, &unknown_pattern),
        None
    );
}

#[test]
fn uniqueness_is_exact_for_strings_and_unknown_array_shapes_decline() {
    let rule = json!({ "type": "array", "uniqueItems": true });
    for (value, expected) in [
        (json!([]), Some(true)),
        (json!(["row", "Row"]), Some(true)),
        (json!(["雪🌸", "other"]), Some(true)),
        (json!(["row", "row"]), Some(false)),
        (json!(["雪🌸", "雪🌸"]), Some(false)),
        (json!([1, 2]), None),
        (json!(["row", null]), None),
        (json!([{}, {}]), None),
    ] {
        assert_eq!(check(&value, &rule, &rule), expected, "{value}");
    }
    let unknown_unique = json!({ "type": "array", "uniqueItems": "true" });
    assert_eq!(check(&json!([]), &unknown_unique, &unknown_unique), None);
    let optional_unique = json!({ "type": "array", "uniqueItems": false });
    assert_eq!(
        check(&json!([1, 1]), &optional_unique, &optional_unique),
        Some(true)
    );
}

#[test]
fn actual_pattern_values_decline_but_empty_and_legacy_options_keep_the_literal_path() {
    let schema = schema().unwrap();
    for (ignore, expected) in [
        (json!(["update:"]), None),
        (json!([""]), None),
        (json!([]), Some(true)),
    ] {
        let value = json!({ "linter": {
            "ruleOptions": { "vue/v-on-event-hyphenation": { "ignore": ignore } },
        } });
        assert_eq!(check(&value, schema, schema), expected);
    }
    for (order, expected) in [
        (json!(["defineProps", "defineEmits"]), Some(true)),
        (json!(["defineProps", "defineProps"]), Some(false)),
    ] {
        let value: Value = json!({ "linter": {
            "ruleOptions": { "script/define-macros-order": { "order": order } },
        } });
        assert_eq!(check(&value, schema, schema), expected);
    }
}
