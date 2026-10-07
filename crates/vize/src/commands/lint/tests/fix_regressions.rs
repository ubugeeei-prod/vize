use super::super::fix::{apply_lint_fixes, fix_until_stable, lint_source_with_optional_fix};
use std::{fs, path::Path};
use vize_l0::{String, cstr};
use vize_patina::{Fix, LintDiagnostic, LintPreset, LintResult, Linter, TextEdit};

#[test]
fn apply_lint_fixes_applies_existing_rule_fixes() {
    let source = r#"<template><button v-on:click="save">Save</button></template>"#;
    let result = Linter::new().lint_sfc(source, "App.vue");
    let fixed = apply_lint_fixes(source, &result).expect("fix should be available");

    assert_eq!(
        fixed.as_str(),
        r#"<template><button @click="save">Save</button></template>"#
    );
}

fn replacement_result(source: &str, replacement: &str) -> LintResult {
    LintResult {
        filename: "App.vue".into(),
        diagnostics: vec![
            LintDiagnostic::warn("test/replacement", source, 0, source.len() as u32).with_fix(
                Fix::new(
                    "Replace",
                    TextEdit::replace(0, source.len() as u32, replacement),
                ),
            ),
        ],
        error_count: 0,
        warning_count: 1,
    }
}

#[test]
fn overlapping_binding_fixes_converge_in_one_invocation() {
    let source = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.vue.txt"
    );
    let expected = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.fixed.vue.txt"
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("TitleBox.vue");
    fs::write(&path, source).unwrap();
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let (fixed, result, changed) =
        lint_source_with_optional_fix(&linter, &path, source.into(), "TitleBox.vue", true).unwrap();
    assert_eq!(fixed.as_str(), expected);
    assert!(changed);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
    assert!(result.diagnostics.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), expected);

    let (again, result, changed) =
        lint_source_with_optional_fix(&linter, &path, fixed, "TitleBox.vue", true).unwrap();
    assert_eq!(again.as_str(), expected);
    assert!(!changed);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn disabled_fix_retains_original_source_and_diagnostics_without_writing() {
    let source = "<template><div v-bind:title=\"title\" /></template>";
    let (unchanged, result, changed) = lint_source_with_optional_fix(
        &Linter::with_preset(LintPreset::Opinionated),
        Path::new("missing/TitleBox.vue"),
        source.into(),
        "TitleBox.vue",
        false,
    )
    .unwrap();
    assert_eq!(unchanged.as_str(), source);
    assert!(!changed);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "vue/v-bind-style")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "vue/prefer-props-shorthand")
    );
}

#[test]
fn no_op_fix_does_not_relint() {
    let initial = replacement_result("same", "same");
    let (source, result) = fix_until_stable("same".into(), initial, |_| panic!("no-op must stop"));
    assert_eq!(source.as_str(), "same");
    assert_eq!(result.diagnostics[0].message, "same");
}

#[test]
fn absent_fixes_do_not_relint() {
    let mut initial = replacement_result("same", "same");
    initial.diagnostics[0].fix = None;
    let (source, result) = fix_until_stable("same".into(), initial, |_| panic!("no fix must stop"));
    assert_eq!(source.as_str(), "same");
    assert!(result.diagnostics[0].fix.is_none());
}

#[test]
fn cyclic_fixes_restore_original_source_and_diagnostics() {
    let mut calls = 0;
    let (source, result) = fix_until_stable("a".into(), replacement_result("a", "b"), |source| {
        calls += 1;
        replacement_result(source, "a")
    });
    assert_eq!(calls, 1);
    assert_eq!(source.as_str(), "a");
    assert_eq!(result.diagnostics[0].message, "a");
    assert_eq!(
        result.diagnostics[0].fix.as_ref().unwrap().edits[0].new_text,
        "b"
    );
}

#[test]
fn nonconvergent_fixes_stop_after_ten_passes_and_report_final_source() {
    let mut calls = 0;
    let (source, result) = fix_until_stable("0".into(), replacement_result("0", "1"), |source| {
        calls += 1;
        let next: String = cstr!("{}", source.parse::<usize>().unwrap() + 1);
        replacement_result(source, &next)
    });
    assert_eq!(calls, 10);
    assert_eq!(source.as_str(), "10");
    assert_eq!(result.diagnostics[0].message, "10");
    assert_eq!(
        result.diagnostics[0].fix.as_ref().unwrap().edits[0].new_text,
        "11"
    );
}
