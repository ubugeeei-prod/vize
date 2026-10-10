//! Complete legacy source API packets for the bounded n8n slot-validity fix.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale, Severity};

const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/slot-directive-validity-8142/cases.json"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/slot-directive-validity-8142/config.json"
);
const DYNAMIC: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/slot-dynamic-bindings-8142/additions.json"
);
const MODIFIERS: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/slot-dynamic-modifiers-8142/additions.json"
);
const HEADER_CHANGES: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/slot-parameter-bindings-8142/legacy-header-changes.json"
);

fn change_header_packet(case: &Value, packet: &mut Value, inverse: bool) -> bool {
    let changes: Value = serde_json::from_str(HEADER_CHANGES).unwrap();
    assert_eq!(changes["changes"].as_array().unwrap().len(), 2);
    let Some(change) = changes["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == case["id"])
    else {
        return false;
    };
    assert_eq!(change["sourceSha256"], case["sourceSha256"]);
    assert_eq!(change["removed"], case["before"]["diagnostics"]);
    let (removed, added) = if inverse {
        (&change["added"], &change["removed"])
    } else {
        (&change["removed"], &change["added"])
    };
    for diagnostic in removed.as_array().unwrap() {
        let diagnostics = packet["diagnostics"].as_array_mut().unwrap();
        let index = diagnostics
            .iter()
            .position(|actual| actual == diagnostic)
            .unwrap();
        diagnostics.remove(index);
    }
    for diagnostic in &change["added"].as_array().unwrap()[..] {
        let source = case["source"].as_str().unwrap();
        let start = diagnostic["start"].as_u64().unwrap() as usize;
        let end = diagnostic["end"].as_u64().unwrap() as usize;
        assert_eq!(source.get(start..end), change["authored"].as_str());
        assert_eq!(
            source.find(change["authored"].as_str().unwrap()),
            Some(start)
        );
    }
    packet["diagnostics"]
        .as_array_mut()
        .unwrap()
        .extend(added.as_array().unwrap().iter().cloned());
    packet["diagnostics"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(|d| (d["start"].as_u64(), d["end"].as_u64()));
    packet["error_count"] = json!(
        packet["error_count"].as_u64().unwrap() - removed.as_array().unwrap().len() as u64
            + added.as_array().unwrap().len() as u64
    );
    true
}

fn modifier_additions(case: &Value) -> Vec<Value> {
    let additions: Value = serde_json::from_str(MODIFIERS).unwrap();
    assert_eq!(additions["additions"].as_array().unwrap().len(), 3);
    additions["additions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|addition| addition["id"] == case["id"])
        .map(|addition| {
            assert_eq!(addition["sourceSha256"], case["sourceSha256"]);
            let diagnostic = addition["diagnostic"].clone();
            let start = diagnostic["start"].as_u64().unwrap() as usize;
            let end = diagnostic["end"].as_u64().unwrap() as usize;
            assert_eq!(
                case["source"].as_str().unwrap().get(start..end),
                addition["authored"].as_str()
            );
            diagnostic
        })
        .collect()
}

fn dynamic_addition(case: &Value) -> Option<Value> {
    let additions: Value = serde_json::from_str(DYNAMIC).unwrap();
    assert_eq!(additions["additions"].as_array().unwrap().len(), 8);
    let addition = additions["additions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == case["id"])?;
    assert_eq!(addition["sourceSha256"], case["sourceSha256"]);
    let diagnostic = addition["diagnostic"].clone();
    let source = case["source"].as_str().unwrap();
    let start = diagnostic["start"].as_u64().unwrap() as usize;
    let end = diagnostic["end"].as_u64().unwrap() as usize;
    assert_eq!(source.get(start..end), addition["authored"].as_str());
    Some(diagnostic)
}

fn current_packet(case: &Value, allow: bool) -> Value {
    let mut packet = case["after"].clone();
    change_header_packet(case, &mut packet, false);
    if let Some(addition) = dynamic_addition(case) {
        let diagnostics = packet["diagnostics"].as_array_mut().unwrap();
        diagnostics.push(addition);
        diagnostics.sort_by_key(|d| (d["start"].as_u64(), d["end"].as_u64()));
        packet["error_count"] = json!(packet["error_count"].as_u64().unwrap() + 1);
    }
    if !allow {
        let added = modifier_additions(case);
        packet["error_count"] = json!(packet["error_count"].as_u64().unwrap() + added.len() as u64);
        let diagnostics = packet["diagnostics"].as_array_mut().unwrap();
        diagnostics.extend(added);
        diagnostics.sort_by_key(|d| (d["start"].as_u64(), d["end"].as_u64()));
    }
    packet
}

fn configured_linter() -> Linter {
    let config: Value = serde_json::from_str(CONFIG).unwrap();
    let rules = config["linter"]["rules"].as_object().unwrap();
    assert_eq!(rules.len(), 51);
    assert_eq!(
        config["linter"]["ruleOptions"].as_object().unwrap().len(),
        3
    );
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
    json!({
        "filename": result.filename.as_str(),
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity,
            "message": d.message.as_str(), "start": d.start, "end": d.end,
            "help": d.help.as_ref().map(|help| help.as_str()),
            "labels": d.labels.iter().map(|l| json!({"message": l.message.as_str(), "start": l.start, "end": l.end})).collect::<Vec<_>>(),
            "fix": d.fix,
        })).collect::<Vec<_>>(),
        "error_count": result.error_count, "warning_count": result.warning_count,
    })
}

#[test]
fn complete_packets_match_twice_and_preserve_all_foreign_findings() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 47);
    let mut captures = Vec::new();
    for allow in [false, true] {
        let linter = configured_linter().with_valid_v_slot_allow_modifiers(allow);
        for case in cases {
            let source = case["source"].as_str().unwrap();
            let filename = case["filename"].as_str().unwrap();
            for _repeat in [1, 2] {
                let actual = complete(&linter.lint_sfc(source, filename));
                captures.push(json!({ "id": case["id"], "allowModifiers": allow,
                    "iteration": _repeat, "actual": actual,
                    "expected": current_packet(case, allow) }));
            }
        }
    }
    let artifact = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/slot-parameter-legacy-packets.json");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(artifact, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    assert_eq!(captures.len(), 188);
    for capture in captures {
        assert_eq!(capture["actual"], capture["expected"], "{}", capture["id"]);
    }
}

#[test]
fn only_declared_diagnostics_are_added_at_exact_authored_byte_ranges() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    for allow in [false, true] {
        let linter = configured_linter().with_valid_v_slot_allow_modifiers(allow);
        let mut additions = 0;
        let mut unchanged = 0;
        let mut dynamic_additions = 0;
        let mut modifier_count = 0;
        let mut header_changes = 0;
        for case in corpus["cases"].as_array().unwrap() {
            let source = case["source"].as_str().unwrap();
            let mut actual = complete(&linter.lint_sfc(source, case["filename"].as_str().unwrap()));
            header_changes += usize::from(change_header_packet(case, &mut actual, true));
            if !allow {
                for expected in modifier_additions(case) {
                    let diagnostics = actual["diagnostics"].as_array_mut().unwrap();
                    let index = diagnostics.iter().position(|d| d == &expected).unwrap();
                    diagnostics.remove(index);
                    actual["error_count"] = json!(actual["error_count"].as_u64().unwrap() - 1);
                    modifier_count += 1;
                }
            }
            if let Some(expected) = dynamic_addition(case) {
                let diagnostics = actual["diagnostics"].as_array_mut().unwrap();
                let index = diagnostics.iter().position(|d| d == &expected).unwrap();
                diagnostics.remove(index);
                actual["error_count"] = json!(actual["error_count"].as_u64().unwrap() - 1);
                dynamic_additions += 1;
            }
            let added = case["added"].as_array().unwrap();
            for expected in added {
                let diagnostics = actual["diagnostics"].as_array_mut().unwrap();
                let index = diagnostics.iter().position(|d| d == expected).unwrap();
                let d = diagnostics.remove(index);
                assert_eq!(d["rule_name"], "vue/valid-v-slot");
                assert_eq!(d["severity"], "error");
                assert!(d["fix"].is_null());
                assert_eq!(d["labels"], json!([]));
                let start = d["start"].as_u64().unwrap() as usize;
                let end = d["end"].as_u64().unwrap() as usize;
                let authored = case["addedDirective"].as_str().unwrap();
                assert_eq!(source.get(start..end), Some(authored), "{}", case["id"]);
                assert_eq!(source.find(authored), Some(start), "{}", case["id"]);
                actual["error_count"] = json!(actual["error_count"].as_u64().unwrap() - 1);
                additions += 1;
            }
            unchanged += usize::from(added.is_empty());
            assert_eq!(actual, case["before"], "{}", case["id"]);
        }
        assert_eq!(additions, 8);
        assert_eq!(unchanged, 39);
        assert_eq!(dynamic_additions, 8);
        assert_eq!(modifier_count, if allow { 0 } else { 3 });
        assert_eq!(header_changes, 2);
    }
}

#[test]
fn isolated_template_rule_reports_complete_localized_packets_without_semantic_demand() {
    let locales = [
        (
            Locale::En,
            "Default v-slot on a component requires a slot parameter value",
            "Add a slot parameter value or use a child <template #default> without a value",
        ),
        (
            Locale::Ja,
            "コンポーネントのデフォルトv-slotにはスロット引数の値が必要です",
            "スロット引数の値を指定するか、値を持たない子要素の<template #default>を使用してください",
        ),
        (
            Locale::Zh,
            "组件上的默认v-slot需要插槽参数值",
            "添加插槽参数值，或使用不带值的子元素<template #default>",
        ),
    ];
    for (locale, message, help) in locales {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_locale(locale)
            .with_help_level(HelpLevel::Full);
        let actual = complete(
            &linter.lint_template("<FancyPanel v-slot>ready</FancyPanel>", "SlotValue.vue"),
        );
        assert_eq!(
            actual,
            json!({
                "filename": "SlotValue.vue", "error_count": 1, "warning_count": 0,
                "diagnostics": [{
                    "rule_name": "vue/valid-v-slot", "severity": "error",
                    "message": message, "start": 12, "end": 18, "help": help,
                    "labels": [], "fix": null,
                }],
            })
        );
    }
}
