//! Authored syntax-only whole packets alongside the unchanged n8n 51-rule corpus.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

const HELP: &str = "Remove the modifiers or enable allowModifiers";

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
fn complete_configured_modifier_packets_repeat_with_unchanged_neighbors() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/slot-dynamic-modifiers-8142/controls.json"
    ))
    .unwrap();
    assert_eq!(cases["cases"].as_array().unwrap().len(), 12);
    for (allow, key) in [(false, "false"), (true, "true")] {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_valid_v_slot_allow_modifiers(allow)
            .with_help_level(HelpLevel::Full);
        for repeat in [1, 2] {
            let mut additions = 0;
            for case in cases["cases"].as_array().unwrap() {
                let source = case["source"].as_str().unwrap();
                let actual = complete(&linter.lint_sfc(source, "DynamicSlot.vue"));
                assert_eq!(
                    actual, case["expectedByOption"][key],
                    "{} allow {allow} repeat {repeat}",
                    case["id"]
                );
                let mut projection = actual.clone();
                for added in case["added"].as_array().unwrap().iter().filter(|_| !allow) {
                    let start = added["start"].as_u64().unwrap() as usize;
                    let end = added["end"].as_u64().unwrap() as usize;
                    assert_eq!(source.get(start..end), case["authored"].as_str());
                    let diagnostics = projection["diagnostics"].as_array_mut().unwrap();
                    let index = diagnostics.iter().position(|d| d == added).unwrap();
                    diagnostics.remove(index);
                    projection["error_count"] =
                        json!(projection["error_count"].as_u64().unwrap() - 1);
                    additions += 1;
                }
                assert_eq!(
                    projection, case["declaredParentProjection"],
                    "{}",
                    case["id"]
                );
            }
            assert_eq!(additions, if allow { 0 } else { 9 });
        }
    }
}

#[test]
fn dynamic_modifier_diagnostic_is_localized() {
    let source =
        "<FancyPanel><template #[name].modifier=\"{ item }\">ready</template></FancyPanel>";
    for (locale, message, help) in [
        (
            Locale::En,
            "v-slot with a slot argument does not support modifiers by default",
            HELP,
        ),
        (
            Locale::Ja,
            "引数を持つv-slotでは既定で修飾子を使用できません",
            "修飾子を削除するかallowModifiersを有効にしてください",
        ),
        (
            Locale::Zh,
            "带插槽参数的v-slot默认不支持修饰符",
            "移除修饰符或启用allowModifiers",
        ),
    ] {
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_locale(locale)
            .with_help_level(HelpLevel::Full)
            .lint_template(source, "DynamicSlot.vue");
        assert_eq!(
            complete(&result),
            json!({
                "filename": "DynamicSlot.vue", "error_count": 1, "warning_count": 0,
                "diagnostics": [{"rule_name": "vue/valid-v-slot", "severity": "error", "message": message,
                  "start": 22, "end": 49, "help": help, "labels": [], "fix": null}],
            })
        );
    }
}

#[test]
fn configuration_does_not_enable_unselected_or_disabled_rules() {
    let source =
        "<template><FancyPanel><template #header.memo>ready</template></FancyPanel></template>";
    for allow in [false, true] {
        let unselected = Linter::with_preset(LintPreset::Incremental)
            .with_valid_v_slot_allow_modifiers(allow)
            .lint_sfc(source, "DynamicSlot.vue");
        let disabled = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_disabled_rules(vec!["vue/valid-v-slot".into()])
            .with_valid_v_slot_allow_modifiers(allow)
            .lint_sfc(source, "DynamicSlot.vue");
        for result in [unselected, disabled] {
            assert_eq!(
                complete(&result),
                json!({
                    "filename": "DynamicSlot.vue", "diagnostics": [], "error_count": 0, "warning_count": 0,
                })
            );
        }
    }
}
