//! Whole source API regression packets for the explicit n8n rule projection.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/lint/bound-slot-attribute/cases.json");
const CONFIG: &str =
    include_str!("../../../tests/_fixtures/differential/lint/bound-slot-attribute/config.json");

fn configured_linter() -> Linter {
    let config: Value = serde_json::from_str(CONFIG).unwrap();
    let rules = config["linter"]["rules"].as_object().unwrap();
    assert_eq!(rules.len(), 51);
    assert!(rules.values().all(|severity| severity == "error"));
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(
            rules.keys().map(|name| name.as_str().into()).collect(),
        ))
        .with_rule_severity_overrides(
            rules
                .keys()
                .map(|name| (name.as_str().into(), Severity::Error))
                .collect(),
        )
        .with_help_level(HelpLevel::Full)
        .with_attribute_hyphenation(vize_patina::rules::HyphenationStyle::Always)
        .with_component_name_in_template_casing(vize_patina::rules::ComponentCasing::PascalCase)
        .with_sfc_element_order_options(vize_patina::rules::SfcElementOrderOptions {
            order: ["script", "template", "style"]
                .into_iter()
                .map(|name| vize_patina::rules::SfcElementOrderGroup::new(vec![name.into()]))
                .collect(),
        })
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|d| {
            let labels: Vec<_> = d
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message.as_str(), "start": label.start, "end": label.end})
                })
                .collect();
            json!({
                "rule_name": d.rule_name, "severity": d.severity,
                "message": d.message.as_str(), "start": d.start, "end": d.end,
                "help": d.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": d.fix,
            })
        })
        .collect();
    json!({
        "filename": result.filename.as_str(), "diagnostics": diagnostics,
        "error_count": result.error_count, "warning_count": result.warning_count,
    })
}

#[test]
fn full_source_packets_match_twice_including_every_foreign_finding() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 46);
    let linter = configured_linter();
    let mut mismatches = Vec::new();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for repeat in [1, 2] {
            let actual = complete(&linter.lint_sfc(source, filename));
            if actual != case["native"] {
                mismatches.push(json!({
                    "id": case["id"], "repeat": repeat,
                    "actual": actual, "expected": case["native"],
                }));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "Complete source packet mismatches: {mismatches:#?}"
    );
}

#[test]
fn new_bound_findings_keep_the_authored_key_and_existing_native_metadata() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let linter = configured_linter();
    let mut checked = 0;
    for case in corpus["cases"].as_array().unwrap() {
        let Some(key) = case["newBoundKey"].as_str() else {
            continue;
        };
        let source = case["source"].as_str().unwrap();
        let result = linter.lint_sfc(source, case["filename"].as_str().unwrap());
        let finding = result.diagnostics.iter().find(|d| {
            d.rule_name == "vue/no-deprecated-slot-attribute"
                && &source[d.start as usize..d.end as usize] == key
        });
        let finding = finding.unwrap_or_else(|| panic!("Missing bound key for {}", case["id"]));
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(
            finding.message,
            "the `slot` attribute was deprecated in Vue 2.6 and removed in Vue 3"
        );
        assert_eq!(
            finding.help.as_deref(),
            Some("Use `v-slot` instead (e.g. `<template v-slot:header>`).")
        );
        assert!(finding.labels.is_empty());
        assert!(finding.fix.is_none());
        checked += 1;
    }
    assert_eq!(checked, 22);
}

#[test]
fn vue2_retains_static_and_bound_slot_attributes() {
    let linter = configured_linter().with_vue_version(Some(vize_l0::config::VueVersion::V2));
    for attribute in ["slot=\"header\"", ":slot=\"name\"", "v-bind:slot=\"name\""] {
        let source = format!(
            "<script>export default {{ components: {{ FancyPanel: {{}} }} }}</script>\n\
             <template><FancyPanel><div {attribute}>Title</div></FancyPanel></template>"
        );
        assert_eq!(
            complete(&linter.lint_sfc(&source, "LegacySlots.vue")),
            json!({
                "filename": "LegacySlots.vue", "error_count": 1, "warning_count": 0,
                "diagnostics": [{
                    "rule_name": "vue/require-component-registration", "severity": "error",
                    "message": "Component is used but not explicitly imported", "start": 78, "end": 88,
                    "help": "Import the component in <script setup> or register it in components option",
                    "labels": [], "fix": null,
                }],
            }),
        );
    }
}
