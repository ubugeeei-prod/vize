//! Authored syntax-only whole packets alongside the unchanged n8n 51-rule corpus.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

const MESSAGE: &str = "Dynamic slot name cannot reference a binding declared by the same slot";
const HELP: &str = "Use a binding from the enclosing scope to choose the slot name";

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
fn complete_dynamic_binding_and_outer_scope_packets() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
        .with_help_level(HelpLevel::Full);
    let cases: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/slot-dynamic-bindings-8142/controls.json"
    ))
    .unwrap();
    assert_eq!(cases["cases"].as_array().unwrap().len(), 16);
    for case in cases["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        for _repeat in [1, 2] {
            let result = linter.lint_sfc(source, "DynamicSlot.vue");
            let actual = complete(&result);
            assert_eq!(actual, case["expected"], "{}", case["id"]);
        }
    }
}

#[test]
fn syntax_only_dynamic_scope_diagnostic_is_localized() {
    let source = "<FancyPanel><template #[slot.name]=\"slot\">ready</template></FancyPanel>";
    for (locale, message, help) in [
        (Locale::En, MESSAGE, HELP),
        (
            Locale::Ja,
            "動的スロット名は同じスロットで宣言された変数を参照できません",
            "スロット名には外側のスコープの変数を使用してください",
        ),
        (
            Locale::Zh,
            "动态插槽名称不能引用同一插槽声明的绑定",
            "使用外层作用域的绑定来选择插槽名称",
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
                "diagnostics": [{
                    "rule_name": "vue/valid-v-slot", "severity": "error",
                    "message": message, "start": 22, "end": 41, "help": help,
                    "labels": [], "fix": null,
                }],
            })
        );
    }
}
