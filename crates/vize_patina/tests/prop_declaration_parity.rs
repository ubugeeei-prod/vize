//! Minimized release Matrix regressions; ranges are independently transcribed
//! from eslint-plugin-vue 10.9.2's original complete declarations/expressions.
use serde::Deserialize;
use vize_patina::{LintPreset, Linter};

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    rule: String,
    severity: String,
    start: u32,
    end: u32,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    diagnostics: Vec<Finding>,
}
#[derive(Deserialize)]
struct Corpus {
    provider: String,
    cases: Vec<Case>,
}

#[test]
fn complete_prop_declaration_diagnostics_match_the_provider() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/prop-declarations-448/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.provider, "eslint-plugin-vue@10.9.2");
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "vue/no-mutating-props".into(),
        "vue/prop-name-casing".into(),
        "script/no-required-prop-with-default".into(),
        "script/require-default-prop".into(),
    ]));
    for mut case in corpus.cases {
        let result = linter.lint_sfc(&case.source, "Probe.vue");
        let mut actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|d| Finding {
                rule: d.rule_name.to_owned(),
                severity: if d.severity == vize_patina::Severity::Error {
                    "error"
                } else {
                    "warning"
                }
                .into(),
                start: d.start,
                end: d.end,
            })
            .collect();
        actual.sort();
        case.diagnostics.sort();
        assert_eq!(actual, case.diagnostics, "{}: {}", case.id, case.source);
    }
}
