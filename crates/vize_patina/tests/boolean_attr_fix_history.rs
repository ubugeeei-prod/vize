//! Issue #7905: complete legacy boolean-attribute findings and authored edits.

use serde::Deserialize;
use vize_patina::{
    Fix, HelpLevel, JsxLang, LintDiagnostic, LintPreset, LintResult, Linter, Locale, TextEdit,
};

const RULE: &str = "vue/no-boolean-attr-value";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    target: String,
    name: String,
    value: String,
    fix: bool,
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

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
}

fn query(linter: &Linter, case: &Case, source: &str) -> LintResult {
    match case.entry.as_str() {
        "sfc" => linter.lint_sfc(source, &case.filename),
        "html" => linter.lint_standalone_html(source, &case.filename),
        "jsx" => linter.lint_jsx(source, &case.filename, JsxLang::Jsx),
        "tsx" => linter.lint_jsx(source, &case.filename, JsxLang::Tsx),
        _ => panic!("unregistered public entry: {}", case.entry),
    }
}

fn expected(case: &Case, help_level: HelpLevel) -> LintResult {
    let diagnostics = case
        .diagnostics
        .iter()
        .map(|finding| {
            assert_eq!(
                case.source.matches(finding.target.as_str()).count(),
                1,
                "{}",
                case.id
            );
            let start = case.source.find(finding.target.as_str()).unwrap() as u32;
            let end = start + finding.target.len() as u32;
            let help = format!(
                "Remove the value. Use just {} instead of {}=\"...\".",
                finding.name, finding.name
            );
            let mut diagnostic = LintDiagnostic::warn(
                RULE,
                format!(
                    "Boolean attribute \"{}\" should not have value \"{}\"",
                    finding.name, finding.value
                ),
                start,
                end,
            );
            if let Some(processed) = help_level.process(&help) {
                diagnostic = diagnostic.with_help(processed);
            }
            if finding.fix {
                diagnostic = diagnostic.with_fix(Fix::new(
                    help,
                    TextEdit::delete(start + finding.name.len() as u32, end),
                ));
            }
            diagnostic
        })
        .collect::<Vec<_>>();
    LintResult {
        filename: case.filename.as_str().into(),
        error_count: 0,
        warning_count: diagnostics.len(),
        diagnostics,
    }
}

fn assert_whole(actual: &LintResult, expected: &LintResult, id: &str) {
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"), "{id}");
}

#[test]
fn original_and_authored_public_entries_preserve_whole_results_and_fixed_bytes() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/issue-7905/cases.json")).unwrap();
    assert_eq!(cases.len(), 21, "every authored vector remains registered");
    assert_eq!(
        cases[0].source,
        include_str!("fixtures/issue-7905/CardList.vue.fixture")
    );
    for case in cases {
        let checker = linter();
        let initial = query(&checker, &case, &case.source);
        assert_whole(&initial, &expected(&case, HelpLevel::Full), &case.id);
        let edits = initial
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.fix.as_ref())
            .flat_map(|fix| fix.edits.iter().cloned())
            .collect();
        let fixed = Fix::with_edits("Apply independent authored edits", edits).apply(&case.source);
        assert_eq!(fixed.as_str(), case.fixed, "{}", case.id);
        let after_expected = if case.diagnostics.iter().any(|finding| finding.fix) {
            LintResult {
                filename: case.filename.as_str().into(),
                diagnostics: vec![],
                error_count: 0,
                warning_count: 0,
            }
        } else {
            expected(&case, HelpLevel::Full)
        };
        for _ in 0..3 {
            let after = query(&checker, &case, &fixed);
            assert_whole(&after, &after_expected, &case.id);
            assert!(
                after
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.fix.is_none())
            );
        }
        // Suppressing help must not suppress the mechanical edit itself.
        let quiet = linter().with_help_level(HelpLevel::None);
        assert_whole(
            &query(&quiet, &case, &case.source),
            &expected(&case, HelpLevel::None),
            &case.id,
        );
    }
}
