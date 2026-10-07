//! Public SFC regression corpus for #7899 and #7901; no Davinci parity claim.
use serde::Deserialize;
use std::path::Path;
use vize_l0::String;
use vize_patina::{HelpLevel, LintPreset, Linter, Locale};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    fixture: String,
    rules: Vec<String>,
    warnings: usize,
}

#[test]
fn derived_watchers_preserve_effects_editable_copies_and_required_markers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/derived-watchers-7901");
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/linter/derived-watchers-7901/cases.json"
    ))
    .expect("strict corpus schema");
    assert_eq!(
        cases.len(),
        13,
        "every original and inverse fixture remains registered"
    );
    for case in cases {
        let source = std::fs::read_to_string(root.join(&case.fixture)).expect("pinned fixture");
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(
                case.rules.iter().map(|rule| rule.as_str().into()).collect(),
            ))
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let filename = case.fixture.trim_end_matches(".txt");
        let result = linter.lint_sfc(&source, filename);
        assert_eq!(
            result.warning_count, case.warnings,
            "{}: {result:#?}",
            case.id
        );
        assert_eq!(result.error_count, 0, "{}: {result:#?}", case.id);
        assert_eq!(result.diagnostics.len(), case.warnings, "{}", case.id);
        for diagnostic in &result.diagnostics {
            assert_eq!(
                diagnostic.rule_name, "script/prefer-computed",
                "{}",
                case.id
            );
            assert!(diagnostic.fix.is_none(), "advice offers no unsafe rewrite");
        }
        assert_eq!(
            format!("{result:#?}"),
            format!("{:#?}", linter.lint_sfc(&source, filename)),
            "{} stable full result",
            case.id
        );
    }
}
