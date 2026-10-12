//! Whole pinned inputs and independent default-order controls for #6881.

use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/attribute-order-spreads/cases.json");
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/attribute-order-spreads/original-matrix-observations.json"
);
const RULE: &str = "vue/attribute-order";

fn configured() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(vize_patina::rules::vue::AttributeOrder));
    Linter::with_registry(registry)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::None)
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message, "start": label.start, "end": label.end})
                })
                .collect();
            json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message, "start": diagnostic.start, "end": diagnostic.end,
                "help": diagnostic.help, "labels": labels, "fix": diagnostic.fix})
        })
        .collect();
    json!({"filename": result.filename, "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

#[test]
fn whole_originals_and_controls_match_complete_service_vectors() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 44);
    assert_eq!(corpus["reference"]["version"], "10.9.2");
    assert_eq!(corpus["reference"]["options"], json!([]));
    assert_eq!(
        cases
            .iter()
            .take(9)
            .map(|case| case["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "original-heyui-src-components-tooltip-tooltip.vue",
            "original-tdesign-packages-theme-generator-src-common-components-ColorPicker-index.vue",
            "original-vue-js-modal-src-components-Dialog.vue",
            "original-gridea-src-components-Main.vue",
            "original-naive-ui-src-calendar-demos-enUS-basic.demo.vue",
            "original-naive-ui-src-dynamic-input-demos-enUS-form.demo.vue",
            "original-naive-ui-src-grid-demos-enUS-collapse.demo.vue",
            "original-naive-ui-src-typography-demos-enUS-router-link.demo.vue",
            "original-splayer-src-views-Radio-hot.vue",
        ]
    );
    let linter = configured();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        assert_eq!(Some(source.len() as u64), case["bytes"].as_u64());
        let result = linter.lint_sfc(source, case["filename"].as_str().unwrap());
        assert_eq!(complete(&result), case["service"], "{}", case["id"]);
    }
}

#[test]
fn configured_error_and_disabled_rule_preserve_complete_payloads() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let errors = configured().with_rule_severity_overrides(vec![(RULE.into(), Severity::Error)]);
    let disabled = configured().with_disabled_rules(vec![RULE.into()]);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let mut expected = case["service"].clone();
        expected["error_count"] = expected["warning_count"].clone();
        expected["warning_count"] = json!(0);
        for diagnostic in expected["diagnostics"].as_array_mut().unwrap() {
            diagnostic["severity"] = json!("error");
        }
        assert_eq!(complete(&errors.lint_sfc(source, filename)), expected);
        assert_eq!(
            complete(&disabled.lint_sfc(source, filename)),
            json!({"filename": filename, "error_count": 0,
                "warning_count": 0, "diagnostics": []}),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn captured_original_failure_and_unprovided_native_route_stay_explicit() {
    let original: Value = serde_json::from_str(ORIGINAL).unwrap();
    assert_eq!(original["run"], 38_205_026_873_u64);
    assert_eq!(original["attempt"], 1);
    assert_eq!(
        original["source"],
        "ef34d1a0409fc711a6c5cefcf0708b0cce78f502"
    );
    assert_eq!(original["all22Terminal"], false);
    assert_eq!(original["usableOnlyTotalFP"], 10);
    assert_eq!(original["usableOnlyTotalFN"], 367);
    assert_eq!(
        original["wholeOriginalSourceEntries"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let refusal = configured()
        .lint_native_template("<Comp @change='change' v-bind='attrs' />", "Exact.vue")
        .unwrap_err();
    assert_eq!(
        refusal,
        vize_patina::native::template::NativeTemplateLintRefusal::UnprovidedRule {
            rule: RULE.into()
        }
    );
}
