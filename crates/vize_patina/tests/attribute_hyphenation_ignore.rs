//! Full diagnostics and actual fixed points against the independent ESLint corpus.
use serde_json::{Value, json};
use vize_l0::config::ConfigLintRuleOptions;
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

const RULE: &str = "vue/attribute-hyphenation";
const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/cases.json"
);
const REFERENCE: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/eslint-plugin-vue.original.json"
);
fn source(file: &str) -> &'static str {
    match file {
        "RegisteredCamel.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/RegisteredCamel.vue.txt"
        ),
        "RegisteredHyphen.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/RegisteredHyphen.vue.txt"
        ),
        _ => panic!("unknown pinned attribute source"),
    }
}
fn linter(policy: &Value) -> Linter {
    let options: ConfigLintRuleOptions = serde_json::from_value(json!({RULE:policy})).unwrap();
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(false)
        .with_help_level(HelpLevel::Full)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_config_rule_options(&options)
}
fn packet(result: &LintResult) -> Value {
    json!({"filename":result.filename,"error_count":result.error_count,"warning_count":result.warning_count,
        "diagnostics":result.diagnostics.iter().map(|d|json!({
            "rule_name":d.rule_name,"severity":d.severity,"message":d.message,
            "start":d.start,"end":d.end,"help":d.help,"fix":d.fix,
            "labels":d.labels.iter().map(|label|json!({"message":label.message,"start":label.start,"end":label.end})).collect::<Vec<_>>()
        })).collect::<Vec<_>>()})
}
fn byte_offset(source: &str, utf16: usize) -> usize {
    let mut seen = 0;
    for (offset, character) in source.char_indices() {
        if seen == utf16 {
            return offset;
        }
        seen += character.len_utf16();
        assert!(seen <= utf16, "boundary must not split a surrogate pair");
    }
    assert_eq!(seen, utf16);
    source.len()
}
fn position(source: &str, finding: &Value, line_key: &str, column_key: &str) -> usize {
    let line = finding[line_key].as_u64().unwrap() as usize;
    let column = finding[column_key].as_u64().unwrap() as usize;
    let preceding = source
        .split('\n')
        .take(line - 1)
        .map(|s| s.encode_utf16().count() + 1)
        .sum::<usize>();
    byte_offset(source, preceding + column - 1)
}
fn expected(source: &str, filename: &str, reference: &Value) -> Value {
    let diagnostics = reference["verify"][0]["messages"].as_array().unwrap().iter().map(|finding| {
        let (message, help) = match finding["messageId"].as_str().unwrap() {
            "mustBeHyphenated" => ("Attribute should be hyphenated", "Use kebab-case for attribute names"),
            "cannotBeHyphenated" => ("Attribute should not be hyphenated", "Use camelCase or a single-word attribute name instead of kebab-case."),
            _ => panic!("unknown independent finding"),
        };
        let start = byte_offset(source, finding["fix"]["range"][0].as_u64().unwrap() as usize);
        let end = byte_offset(source, finding["fix"]["range"][1].as_u64().unwrap() as usize);
        let key = &source[start..end];
        // The existing Vize API edits the name, while ESLint edits the complete key.
        let prefix = match key {
            "fooBar" | "foo-bar" => "",
            ":activeKey" | ":active-key" => ":",
            "v-model:pageSize" | "v-model:page-size" => "v-model:",
            _ => panic!("independent corpus key mismatch"),
        };
        let replacement = finding["fix"]["text"].as_str().unwrap().strip_prefix(prefix).unwrap();
        json!({"rule_name":RULE,"severity":"warning","message":message,"help":help,"labels":[],
            "start":position(source,finding,"line","column"),"end":position(source,finding,"endLine","endColumn"),
            "fix":{"message":help,"edits":[{"start":start+prefix.len(),"end":end,"new_text":replacement}]}})
    }).collect::<Vec<_>>();
    json!({"filename":filename,"error_count":0,"warning_count":diagnostics.len(),"diagnostics":diagnostics})
}
fn clean(filename: &str) -> Value {
    json!({"filename":filename,"error_count":0,"warning_count":0,"diagnostics":[]})
}

#[test]
fn canonical_attribute_ignore_preserves_nine_whole_vectors_fixes_and_fixed_points() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    let rows = corpus["valid"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    for row in rows {
        let source = source(row["file"].as_str().unwrap());
        let filename = row["filename"].as_str().unwrap();
        let baseline = &reference["rows"][row["referenceRow"].as_u64().unwrap() as usize];
        assert_eq!(baseline["id"], row["id"]);
        assert_eq!(baseline["source"], source);
        let linter = linter(&row["policy"]);
        let actual = linter.lint_sfc(source, filename);
        assert_eq!(
            packet(&actual),
            expected(source, filename, baseline),
            "{}",
            row["id"]
        );
        let mut edits = actual
            .diagnostics
            .iter()
            .flat_map(|d| d.fix.as_ref().unwrap().edits.iter())
            .collect::<Vec<_>>();
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
        let mut fixed = source.to_owned();
        for edit in edits {
            fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        assert_eq!(
            fixed,
            baseline["fixedSource"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        for _ in 0..2 {
            assert_eq!(packet(&linter.lint_sfc(&fixed, filename)), clean(filename));
        }
        assert_eq!(
            packet(
                &linter
                    .with_enabled_rules(Some(vec![]))
                    .lint_sfc(source, filename)
            ),
            clean(filename)
        );
    }
}

#[test]
fn custom_ignore_extends_builtins_and_keeps_existing_typechecker_exemptions() {
    let input = r#"<template><Widget data-testId="x" aria-label="x" slot-scope="x" viewBox="x" ariaLabel="x" :dataSource="x" customCamel="x" /></template>"#;
    let configured = linter(&json!({"ignore":["custom"]}));
    assert_eq!(
        packet(&configured.lint_sfc(input, "Control.vue")),
        clean("Control.vue")
    );
    let empty = linter(&json!({"ignore":[]})).lint_sfc(input, "Control.vue");
    assert_eq!(empty.diagnostics.len(), 1);
    assert_eq!(
        &input[empty.diagnostics[0].start as usize..empty.diagnostics[0].end as usize],
        r#"customCamel="x""#
    );
    let never = r#"<template><Widget data-test="x" aria-label="x" slot-scope="x" custom-prop="x" /></template>"#;
    assert_eq!(
        packet(
            &linter(&json!({"style":"never","ignore":["custom"]})).lint_sfc(never, "Control.vue")
        ),
        clean("Control.vue")
    );
}
