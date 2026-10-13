//! `<transition>` children the reference rule treats as custom components.

use serde_json::Value;
use vize_patina::{LintPreset, Linter, Severity};

const RULE: &str = "vue/require-toggle-inside-transition";

#[test]
fn only_native_untoggled_children_are_reported_at_their_start_tag() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/linter/transition-custom-element-children/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus["rule"], RULE);
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11);
    for preset in [LintPreset::Essential, LintPreset::HappyPath] {
        let linter = Linter::with_preset(preset);
        for case in cases {
            let source = case["source"].as_str().unwrap();
            let result = linter.lint_sfc(source, case["filename"].as_str().unwrap());
            let actual: Vec<_> = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == RULE)
                .map(|diagnostic| (diagnostic.severity, diagnostic.start, diagnostic.end))
                .collect();
            let expected: Vec<_> = case["reported"]
                .as_str()
                .map(|start_tag| {
                    let start = source.find(start_tag).unwrap();
                    (
                        Severity::Error,
                        u32::try_from(start).unwrap(),
                        u32::try_from(start + start_tag.len()).unwrap(),
                    )
                })
                .into_iter()
                .collect();
            assert_eq!(actual, expected, "{} / {}", case["id"], preset.as_str());
        }
    }
}
