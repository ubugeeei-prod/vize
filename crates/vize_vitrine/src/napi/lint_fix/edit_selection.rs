//! Compare the native entry point with the versioned diagnostic batch.

use super::{apply_lint_fixes, lint_source};
use vize_l0::{SourceRoot, Span, String};
use vize_l1::edit::VersionedSource;
use vize_patina::{Fix, LintDiagnostic, LintPreset, LintResult, Linter, TextEdit};

fn compare(source: &str, version: i32, result: &LintResult) -> Option<String> {
    let frame = VersionedSource::new(
        result.filename.as_str(),
        version,
        SourceRoot::new(source).unwrap(),
    );
    let candidates = result
        .diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .flat_map(|f| &f.edits)
        .map(|e| (Span::new(e.start, e.end), e.new_text.as_str()));
    let applied = frame
        .select_edits(candidates)
        .unwrap()
        .map(|e| e.as_edit_set().apply(frame).unwrap());
    assert_eq!(
        applied,
        apply_lint_fixes(source, result),
        "{}",
        result.filename
    );
    applied
}

#[test]
fn ties_overlaps_and_invalid_ranges_match_the_native_binding() {
    let source = "α🌱z";
    let mut result = LintResult {
        filename: "unicode.vue".into(),
        warning_count: 0,
        error_count: 0,
        diagnostics: vec![],
    };
    assert_eq!(compare(source, 1, &result), None);
    for edits in [
        vec![
            TextEdit::replace(0, 1, "bad"),
            TextEdit::replace(6, 2, "bad"),
            TextEdit::insert(10, "bad"),
        ],
        vec![
            TextEdit::replace(2, 6, "x"),
            TextEdit::insert(2, "1"),
            TextEdit::replace(0, 6, "loses"),
            TextEdit::replace(0, 2, "a"),
        ],
        vec![
            TextEdit::insert(2, "2"),
            TextEdit::replace(2, 6, "loses tie"),
            TextEdit::insert(6, "!"),
        ],
    ] {
        result.diagnostics.push(
            LintDiagnostic::warn("test/fix", "fixture", 0, 0)
                .with_fix(Fix::with_edits("edits", edits)),
        );
        result.warning_count += 1;
        compare(source, 2, &result);
    }
    assert_eq!(compare(source, 3, &result).as_deref(), Some("a12x!z"));
}

#[test]
fn each_native_pass_matches_the_original_vue_fix_history() {
    let before = include_str!(
        "../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.vue.txt"
    );
    let expected = include_str!(
        "../../../../../tests/_fixtures/differential/linter/lint-fix-passes-7906/TitleBox.fixed.vue.txt"
    );
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let mut text = String::from(before);
    for version in 1..=10 {
        let result = lint_source(&linter, &text, "TitleBox.vue");
        match compare(&text, version, &result) {
            Some(next) if next != text => text = next,
            _ => break,
        }
    }
    assert_eq!(text, expected);
    let final_result = lint_source(&linter, &text, "TitleBox.vue");
    assert_eq!(compare(&text, 11, &final_result), None);
}
