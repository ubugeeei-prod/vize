//! Complete #7962 source and diagnostic contracts for actual setup macros.

use std::path::Path;
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const RULE: &str = "script/no-with-defaults";
const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/lint-with-defaults/cases.json");
const HELP: &str = "Use destructuring with defaults: `const { count = 0, name = 'default' } = defineProps<Props>()`";

fn check_cases(ids: &[&str]) {
    let linter =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    let cases: serde_json::Value = serde_json::from_str(CASES).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lint-with-defaults");
    for &id in ids {
        let case = cases
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"].as_str() == Some(id))
            .unwrap();
        let source =
            std::fs::read_to_string(fixture.join(case["input"].as_str().unwrap())).unwrap();
        let filename = case["filename"].as_str().unwrap();
        let result = match case["kind"].as_str().unwrap() {
            "sfc" => linter.lint_sfc(&source, filename),
            "script" => linter.lint_script(&source, filename),
            "html" => linter.lint_standalone_html(&source, filename),
            _ => panic!("unregistered fixture kind"),
        };
        check_result(&result, case);
    }
}

fn check_result(result: &LintResult, case: &serde_json::Value) {
    let spans = case["spans"].as_array().unwrap();
    assert_eq!(result.filename.as_str(), case["filename"].as_str().unwrap());
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, spans.len());
    assert_eq!(result.diagnostics.len(), spans.len(), "{}", case["id"]);
    for (diagnostic, span) in result.diagnostics.iter().zip(spans) {
        assert_eq!(diagnostic.rule_name, RULE);
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.message.as_str(),
            "Prefer destructuring defaults over withDefaults (Vue 3.5+)"
        );
        assert_eq!(u64::from(diagnostic.start), span[0].as_u64().unwrap());
        assert_eq!(u64::from(diagnostic.end), span[1].as_u64().unwrap());
        assert_eq!(diagnostic.help.as_deref(), Some(HELP));
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
}

#[test]
fn original_vue_and_ts_comments_and_strings_have_no_diagnostics() {
    check_cases(&["original-sfc", "original-comment", "original-source-check"]);
}

#[test]
fn actual_setup_macro_owners_keep_complete_original_identifier_diagnostics() {
    check_cases(&[
        "typed-spaced",
        "typed-wrapped",
        "parenthesized-calls",
        "expression-statement",
        "multiple-declarators",
        "escaped-callee",
        "nested-shadow-does-not-hide-macro",
    ]);
}

#[test]
fn invalid_macro_arguments_still_preserve_the_discouraged_macro_finding() {
    check_cases(&["runtime-props-invalid", "missing-defaults-invalid"]);
}

#[test]
fn ordinary_nested_member_optional_and_non_setup_calls_are_not_macros() {
    check_cases(&[
        "ordinary-expressions",
        "classic-script",
        "plain-actual-call",
        "inline-html",
    ]);
}

#[test]
fn original_setup_owner_keeps_physical_unicode_crlf_offsets_in_dual_scripts() {
    check_cases(&["dual-script-unicode-crlf"]);
}
