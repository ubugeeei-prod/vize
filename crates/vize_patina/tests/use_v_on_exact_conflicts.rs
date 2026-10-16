//! Independent reference packets plus two physically proven reference blind spots (#9226).

use serde::Deserialize;
use vize_patina::{HelpLevel, LintPreset, Linter, Locale, Severity};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Finding {
    key: String,
    start: u32,
    end: u32,
}

#[test]
fn complete_originals_and_controls_preserve_real_listener_conflicts_and_key_ranges() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/use-v-on-exact/cases.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 45);
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/use-v-on-exact".into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::None);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let result = linter.lint_sfc(source, case["filename"].as_str().unwrap());
        let actual = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                assert_eq!(diagnostic.rule_name, "vue/use-v-on-exact", "{}", case["id"]);
                assert_eq!(diagnostic.severity, Severity::Warning, "{}", case["id"]);
                assert!(diagnostic.fix.is_none(), "{}", case["id"]);
                Finding {
                    key: source
                        .get(diagnostic.start as usize..diagnostic.end as usize)
                        .unwrap()
                        .into(),
                    start: diagnostic.start,
                    end: diagnostic.end,
                }
            })
            .collect::<Vec<_>>();
        let expected: Vec<Finding> = serde_json::from_value(case["expected"].clone()).unwrap();
        assert_eq!(actual, expected, "{}", case["id"]);
        assert_eq!(result.error_count, 0, "{}", case["id"]);
        assert_eq!(result.warning_count, expected.len(), "{}", case["id"]);
    }
}
