//! Run the proposed versioned selector against the actual CLI batch applier.

use super::super::fix::{apply_lint_fixes, fix_until_stable};
use serde::Deserialize;
use vize_l0::{SourceRoot, Span, String};
use vize_l1::edit::VersionedSource;
use vize_patina::{JsxLang, LintPreset, LintResult, Linter};

fn selected(source: &str, result: &LintResult) -> Option<String> {
    let frame = VersionedSource::new(
        result.filename.as_str(),
        0,
        SourceRoot::new(source).unwrap(),
    );
    let edits = result
        .diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .flat_map(|f| &f.edits)
        .map(|e| (Span::new(e.start, e.end), e.new_text.as_str()));
    let owner = frame.select_edits(edits).unwrap()?;
    Some(owner.as_edit_set().apply(frame).unwrap())
}

fn compare(source: &str, result: &LintResult) -> Option<String> {
    let value = selected(source, result);
    assert_eq!(
        value,
        apply_lint_fixes(source, result),
        "{}",
        result.filename
    );
    value
}

#[derive(Deserialize)]
struct Case {
    id: String,
    entry: String,
    filename: String,
    source: String,
    fixed: String,
}

#[test]
fn existing_fix_history_replays_cli_edits_and_all_fixed_bytes() {
    for (rule, fixture) in [
        (
            "vue/no-boolean-attr-value",
            include_str!("../../../../../vize_patina/tests/fixtures/issue-7905/cases.json"),
        ),
        (
            "vue/v-slot-style",
            include_str!(
                "../../../../../vize_patina/tests/fixtures/issue-7905-slot-style/cases.json"
            ),
        ),
    ] {
        let checker = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![rule.into()]));
        for case in serde_json::from_str::<Vec<Case>>(fixture).unwrap() {
            let original = match case.entry.as_str() {
                "sfc" => checker.lint_sfc(&case.source, &case.filename),
                "html" => checker.lint_standalone_html(&case.source, &case.filename),
                "jsx" => checker.lint_jsx(&case.source, &case.filename, JsxLang::Jsx),
                "tsx" => checker.lint_jsx(&case.source, &case.filename, JsxLang::Tsx),
                entry => panic!("unknown original entry {entry}"),
            };
            let actual = compare(&case.source, &original).unwrap_or(case.source);
            assert_eq!(actual, case.fixed, "{}", case.id);
        }
    }
    let before = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.vue.txt"
    );
    let expected = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.fixed.vue.txt"
    );
    let checker = Linter::with_preset(LintPreset::Opinionated);
    let check = |text: &str| {
        let result = checker.lint_sfc(text, "TitleBox.vue");
        compare(text, &result);
        result
    };
    let (fixed, _) = fix_until_stable(before.into(), check(before), check);
    assert_eq!(fixed, expected);
}

#[test]
#[ignore = "NUL-delimited list of pinned Vue inputs"]
fn original_vue_fix_batches() {
    let file = std::env::var_os("VIZE_EDIT_SELECTION_MANIFEST").unwrap();
    let manifest = std::fs::read_to_string(file).unwrap();
    let checker = Linter::with_preset(LintPreset::Opinionated);
    let mut files = 0;
    let mut fixed = 0;
    let mut edits = 0;
    let mut small = 0;
    for path in manifest.split_terminator('\0') {
        let path = std::path::Path::new(path);
        let source = davinci_test_support::corpus::read_corpus_source(path);
        let result = checker.lint_sfc(&source, path.to_str().unwrap());
        let count: usize = result
            .diagnostics
            .iter()
            .filter_map(|d| d.fix.as_ref())
            .map(|f| f.edits.len())
            .sum();
        edits += count;
        files += 1;
        if compare(&source, &result).is_some() {
            fixed += 1;
            if count <= 2 {
                small += 1;
            }
        }
    }
    assert!(files > 0 && fixed > 0);
    eprintln!("files={files} fixable={fixed} edits={edits} at_most_two={small}");
}

mod profile;
