//! #7905: authored slot edits, whole original results and UTF-16 rendering.

use serde::Deserialize;
use vize_l0::cstr;
use vize_patina::{
    Fix, HelpLevel, JsxLang, LintDiagnostic, LintPreset, LintResult, Linter, Locale, TextEdit,
    telegraph::LspEmitter,
};

const RULE: &str = "vue/v-slot-style";
const HELP: &str = "**Shorthand:** `<template #header>` (default)\n**Longform:** `<template v-slot:header>`\n\nChoose one style and be consistent.";
const PLAIN_HELP: &str = "Shorthand: <template #header> (default)\nLongform: <template v-slot:header>\nChoose one style and be consistent.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Edit {
    start: u32,
    end: u32,
    new_text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    target: String,
    message: String,
    start: u32,
    end: u32,
    utf16: serde_json::Value,
    edit: Option<Edit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    filename: String,
    entry: String,
    source: String,
    fixed: String,
    diagnostics: Vec<Finding>,
}

fn linter(level: HelpLevel) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(level)
}

fn query(linter: &Linter, case: &Case, source: &str) -> LintResult {
    match case.entry.as_str() {
        "sfc" => linter.lint_sfc(source, &case.filename),
        "html" => linter.lint_standalone_html(source, &case.filename),
        "jsx" => linter.lint_jsx(source, &case.filename, JsxLang::Jsx),
        "tsx" => linter.lint_jsx(source, &case.filename, JsxLang::Tsx),
        _ => panic!("unknown authored host: {}", case.entry),
    }
}

fn expected(case: &Case, level: HelpLevel, after: bool) -> LintResult {
    let diagnostics = case
        .diagnostics
        .iter()
        .filter(|finding| !after || finding.edit.is_none())
        .map(|finding| {
            let mut d =
                LintDiagnostic::warn(RULE, finding.message.as_str(), finding.start, finding.end);
            if let Some(processed) = level.process(HELP) {
                d = d.with_help(processed);
            }
            if let Some(edit) = &finding.edit {
                d = d.with_fix(Fix::new(
                    HELP,
                    TextEdit::replace(edit.start, edit.end, edit.new_text.as_str()),
                ));
            }
            d
        })
        .collect::<Vec<_>>();
    LintResult {
        filename: case.filename.as_str().into(),
        error_count: 0,
        warning_count: diagnostics.len(),
        diagnostics,
    }
}

#[test]
fn original_and_twenty_eight_controls_match_whole_public_results_and_stable_edits() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/issue-7905-slot-style/cases.json")).unwrap();
    assert_eq!(cases.len(), 29);
    assert_eq!(
        cases[0].source,
        include_str!("fixtures/issue-7905/CardList.vue.fixture")
    );
    for case in cases {
        for finding in &case.diagnostics {
            assert_eq!(
                case.source
                    .get(finding.start as usize..finding.end as usize),
                Some(finding.target.as_str()),
                "{}",
                case.id
            );
        }
        for level in [HelpLevel::Full, HelpLevel::None] {
            let checker = linter(level);
            let initial = query(&checker, &case, &case.source);
            assert_eq!(
                cstr!("{initial:?}"),
                cstr!("{:?}", expected(&case, level, false)),
                "{}",
                case.id
            );
            let expected_lsp = case
                .diagnostics
                .iter()
                .map(|finding| {
                    serde_json::json!({
                        "range": finding.utf16, "severity": 2,
                    "message": if level == HelpLevel::Full {
                        cstr!("{}\n{PLAIN_HELP}", finding.message)
                    } else { finding.message.as_str().into() },
                        "source": "vize-patina", "code": RULE,
                    })
                })
                .collect::<Vec<_>>();
            let actual_lsp = LspEmitter::to_lsp_diagnostics_with_source(&initial, &case.source);
            assert_eq!(
                serde_json::to_value(actual_lsp).unwrap(),
                serde_json::json!(expected_lsp),
                "{}",
                case.id
            );
            let edits = initial
                .diagnostics
                .iter()
                .filter_map(|d| d.fix.as_ref())
                .flat_map(|fix| fix.edits.iter().cloned())
                .collect();
            let fixed = Fix::with_edits("authored slot prefix edits", edits).apply(&case.source);
            assert_eq!(fixed.as_str(), case.fixed, "{}", case.id);
            for _ in 0..3 {
                let after = query(&checker, &case, &fixed);
                assert_eq!(
                    cstr!("{after:?}"),
                    cstr!("{:?}", expected(&case, level, true)),
                    "{}",
                    case.id
                );
                assert!(after.diagnostics.iter().all(|d| d.fix.is_none()));
            }
        }
    }
}
