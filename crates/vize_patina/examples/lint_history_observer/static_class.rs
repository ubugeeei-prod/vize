//! Complete existing public static-class repair oracle, without field filtering.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{HelpLevel, LintPreset, Linter, Locale};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Case {
    pub(super) id: String,
    pub(super) entry: String,
    pub(super) source: String,
    pub(super) fixed: String,
    pub(super) diagnostics: usize,
    pub(super) edit_start: Option<u32>,
    pub(super) edit_end: Option<u32>,
    pub(super) diagnostic_start: Option<u32>,
    pub(super) diagnostic_end: Option<u32>,
}

pub(super) fn capture(input: &str) -> Result<String, serde_json::Error> {
    let case: Case = serde_json::from_str(input)?;
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vapor/prefer-static-class".into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let lint = |source: &str| match case.entry.as_str() {
        "template" => linter.lint_template(source, "StaticClass.vue"),
        "sfc" => linter.lint_sfc(source, "StaticClass.vue"),
        entry => panic!("unexpected fixture entry: {entry}"),
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
        let Some(diagnostic) = initial.diagnostics.first() else {
            panic!("missing offered diagnostic: {}", case.id);
        };
        assert_eq!(Some(diagnostic.start), case.diagnostic_start, "{}", case.id);
        assert_eq!(Some(diagnostic.end), case.diagnostic_end, "{}", case.id);
        let [fix] = offers.as_slice() else {
            panic!("missing single offered fix: {}", case.id);
        };
        assert_eq!(fix.edits.len(), 1, "{}", case.id);
        let [edit] = fix.edits.as_slice() else {
            panic!("missing single offered edit: {}", case.id);
        };
        assert_eq!((edit.start, edit.end), (start, end), "{}", case.id);
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
    Ok(format!("{:#?}\n", (&case, &initial, &result)).into())
}
