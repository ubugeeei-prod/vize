//! #7332 and #7328: accessible image names and mutually exclusive landmarks.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{LintPreset, Linter};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    rule: String,
    source: String,
    warnings: usize,
}

#[test]
fn accessible_content_corpus_preserves_real_violations() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "fixtures/accessible-content/cases.json"
    ))
    .expect("strict accessible content corpus");
    assert_eq!(cases.len(), 23);
    for case in cases {
        let linter = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(Some(vec![case.rule.clone()]));
        let result = linter.lint_sfc(&case.source, "AccessibleContent.vue");
        assert_eq!(
            result.error_count, 0,
            "{}: {:?}",
            case.id, result.diagnostics
        );
        assert_eq!(
            result.warning_count, case.warnings,
            "{}: {:?}",
            case.id, result.diagnostics
        );
        assert_eq!(result.diagnostics.len(), case.warnings, "{}", case.id);
        for diagnostic in result.diagnostics {
            assert_eq!(diagnostic.rule_name, case.rule, "{}", case.id);
            assert!(diagnostic.start < diagnostic.end, "{}", case.id);
            assert!(diagnostic.end as usize <= case.source.len(), "{}", case.id);
        }
    }
}
