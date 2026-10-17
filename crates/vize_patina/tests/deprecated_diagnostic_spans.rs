//! Exact filter/tag ranges from pinned Vue ESLint packets and real projects.

use serde_json::Value;
use vize_patina::rules::vue::{NoDeprecatedFilter, NoDeprecatedRouterLinkTagProp};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry};

const ORACLE: &str = include_str!("fixtures/deprecated-diagnostic-spans/oracle.json");

// ESLint columns count UTF-16 code units, while the Rust API uses byte offsets.
fn byte_offset(source: &str, line: &Value, column: &Value) -> u32 {
    let line_index = line.as_u64().unwrap() as usize - 1;
    let column_index = column.as_u64().unwrap() as usize - 1;
    let prefix: usize = source
        .split_inclusive('\n')
        .take(line_index)
        .map(str::len)
        .sum();
    let mut utf16 = 0;
    for (offset, character) in source[prefix..].char_indices() {
        if utf16 == column_index {
            return (prefix + offset) as u32;
        }
        utf16 += character.len_utf16();
    }
    assert_eq!(utf16, column_index);
    source.len() as u32
}

#[test]
fn pinned_originals_and_minimal_cases_match_complete_native_packets() {
    let fixture: Value = serde_json::from_str(ORACLE).unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedFilter));
    registry.register(Box::new(NoDeprecatedRouterLinkTagProp));
    let linter = Linter::with_registry(registry)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let cases = fixture["captures"].as_array().unwrap();
    assert_eq!(cases.len(), 49);
    let mut administrative_warnings = 0;
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let packet = &case["packets"][0];
        if matches!(
            case["id"].as_str().unwrap(),
            "typescript-assertion"
                | "frappe-builder/frontend/src/components/PageClientScriptManager.vue"
        ) {
            // vue-eslint-parser splits the type union before the TS parser.
            // Keep #7911's native correctness: this pipe is in type grammar.
            let (whole_input, union_start, union_end) = if case["id"] == "typescript-assertion" {
                (
                    r#"<template><CodeEditor :type="activeScript.script_type as 'JavaScript' | 'CSS'" /></template>"#,
                    29,
                    77,
                )
            } else {
                (
                    include_str!(
                        "fixtures/deprecated-diagnostic-spans/originals/frappe-builder/frontend/src/components/PageClientScriptManager.vue"
                    ),
                    3814,
                    3862,
                )
            };
            assert_eq!(source, whole_input, "complete original TS-assertion input");
            assert_eq!(
                source.get(union_start..union_end),
                Some("activeScript.script_type as 'JavaScript' | 'CSS'"),
                "the exact source-owned span is a type union, not a filter"
            );
            assert_eq!(packet["errorCount"], 1);
            assert_eq!(packet["messages"][0]["ruleId"], "vue/no-deprecated-filter");
            let actual = linter.lint_sfc(source, filename);
            let expected = LintResult {
                filename: filename.into(),
                diagnostics: vec![],
                error_count: 0,
                warning_count: 0,
            };
            assert_eq!(
                format!("{actual:#?}"),
                format!("{expected:#?}"),
                "{}",
                case["id"]
            );
            continue;
        }
        let mut file_administrative_warnings = 0;
        let diagnostics = packet["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|message| {
                if message["ruleId"].is_null() {
                    // ESLint's host-level unused-disable check is not either
                    // selected Vue rule. Its complete messages and fixes stay
                    // in ORACLE and are replayed by the Node provider test.
                    assert_eq!(message["message"], "Unused eslint-disable directive (no problems were reported from 'no-alert').");
                    assert_eq!(message["severity"], 1);
                    assert!(message["fix"].is_object());
                    file_administrative_warnings += 1;
                    return None;
                }
                let rule = message["ruleId"].as_str().unwrap();
                let (rule, text, help) = match rule {
                    "vue/no-deprecated-filter" => (
                        "vue/no-deprecated-filter",
                        "Filters were removed in Vue 3; replace the '|' filter with a method call or computed property",
                        "Replace the filter with a method call or computed property (e.g. {{ capitalize(message) }} instead of {{ message | capitalize }}).",
                    ),
                    "vue/no-deprecated-router-link-tag-prop" => (
                        "vue/no-deprecated-router-link-tag-prop",
                        "The `tag` prop on `<router-link>` was removed in Vue Router 4; use the v-slot API instead",
                        "Remove the `tag` prop and render the element yourself using `v-slot` (it exposes `href`, `navigate`, and `isActive`).",
                    ),
                    _ => panic!("unexpected provider rule: {rule}"),
                };
                Some(LintDiagnostic::error(
                    rule,
                    text,
                    byte_offset(source, &message["line"], &message["column"]),
                    byte_offset(source, &message["endLine"], &message["endColumn"]),
                )
                .with_help(help))
            })
            .collect();
        let expected = LintResult {
            filename: filename.into(),
            diagnostics,
            error_count: packet["errorCount"].as_u64().unwrap() as usize,
            warning_count: packet["warningCount"].as_u64().unwrap() as usize
                - file_administrative_warnings,
        };
        administrative_warnings += file_administrative_warnings;
        let actual = linter.lint_sfc(source, filename);
        // Compare every public native field, including help, labels and fixes.
        assert_eq!(
            format!("{actual:#?}"),
            format!("{expected:#?}"),
            "{}",
            case["id"]
        );
    }
    assert_eq!(administrative_warnings, 4);
}

#[test]
fn excessive_filter_input_nesting_keeps_the_complete_lexical_finding() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedFilter));
    let linter = Linter::with_registry(registry)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    for depth in [32, 4096] {
        for input in ["value", "value as A | B"] {
            let expression = format!("{}{input}{} | format", "(".repeat(depth), ")".repeat(depth));
            let source = format!("<div>{{{{ {expression} }}}}</div>");
            let start = source.find(&expression).unwrap() as u32;
            let end = start + expression.len() as u32;
            let diagnostic = LintDiagnostic::error(
                "vue/no-deprecated-filter",
                "Filters were removed in Vue 3; replace the '|' filter with a method call or computed property",
                start,
                end,
            ).with_help("Replace the filter with a method call or computed property (e.g. {{ capitalize(message) }} instead of {{ message | capitalize }}).");
            let expected = LintResult {
                filename: "Deep.vue".into(),
                diagnostics: vec![diagnostic],
                error_count: 1,
                warning_count: 0,
            };
            let actual = linter.lint_template(&source, "Deep.vue");
            assert_eq!(
                format!("{actual:#?}"),
                format!("{expected:#?}"),
                "depth {depth}"
            );
        }
    }
}
