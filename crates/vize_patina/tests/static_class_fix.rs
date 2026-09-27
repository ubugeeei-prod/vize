//! #6920: exact public autofix spans and complete corrected template/SFC bytes.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{HelpLevel, LintPreset, Linter, Locale};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    entry: String,
    source: String,
    fixed: String,
    diagnostics: usize,
    edit_start: Option<u32>,
    edit_end: Option<u32>,
    diagnostic_start: Option<u32>,
    diagnostic_end: Option<u32>,
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "Insta captures complete actual Debug fields for the regression corpus"
)]
fn public_static_class_fixes_preserve_every_other_authored_byte() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint-static-class/cases.json"
    ))
    .expect("strict regression corpus schema");
    assert_eq!(cases.len(), 9, "all regression inputs must execute");
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vapor/prefer-static-class".into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    for case in cases {
        let lint = |source: &str| match case.entry.as_str() {
            "template" => linter.lint_template(source, "StaticClass.vue"),
            "sfc" => linter.lint_sfc(source, "StaticClass.vue"),
            entry => panic!("unexpected test entry: {entry}"),
        };
        let initial = lint(&case.source);
        assert_eq!(initial.error_count, 0, "{}", case.id);
        assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id);
        let offers: Vec<_> = initial
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.fix.as_ref())
            .collect();
        let result = if let (Some(start), Some(end)) = (case.edit_start, case.edit_end) {
            assert_eq!(offers.len(), 1, "{}", case.id);
            let diagnostic = &initial.diagnostics[0];
            assert_eq!(Some(diagnostic.start), case.diagnostic_start, "{}", case.id);
            assert_eq!(Some(diagnostic.end), case.diagnostic_end, "{}", case.id);
            let fix = offers[0];
            assert_eq!(fix.edits.len(), 1, "{}", case.id);
            assert_eq!(
                (fix.edits[0].start, fix.edits[0].end),
                (start, end),
                "{}",
                case.id
            );
            let fixed = fix.apply(&case.source);
            assert_eq!(fixed, case.fixed, "{}", case.id);
            let after = lint(&fixed);
            assert_eq!(after.error_count, 0, "{}", case.id);
            assert_eq!(after.warning_count, 0, "{}", case.id);
            assert!(after.diagnostics.is_empty(), "{}", case.id);
            after
        } else {
            assert!(offers.is_empty(), "{}", case.id);
            assert_eq!(case.source, case.fixed, "{}", case.id);
            lint(&case.source)
        };
        // Record complete fields, preserving initial help/labels/fix metadata too.
        insta::assert_debug_snapshot!(case.id.as_str(), (&case, &initial, &result));
    }
}
