use crate::{HelpLevel, LintPreset, LintResult, Linter};
use serde::Deserialize;
use serde_json::{Value, json};
use vize_l0::String;
use vize_l0::config::{ConfigLintRuleOptions, DefineMacrosOrderOptions};

const RULE: &str = "script/define-macros-order";
const FILENAME: &str = "MacroOptions.vue";
const HELP: &str = "Declare `defineProps` and then `defineEmits` first in `<script setup>`, right after imports and type declarations.";
const CORPUS: &str = include_str!(
    "../../../../../tests/_fixtures/differential/linter/macro-order-config-6097/cases.json"
);

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    options: Option<DefineMacrosOrderOptions>,
    statement: Option<String>,
    reference: Reference,
}

#[derive(Deserialize)]
struct Reference {
    messages: Vec<Finding>,
}

#[derive(Deserialize)]
struct Finding {
    message: String,
}

fn source(case: &Case) -> &'static str {
    match case.source.as_str() {
        "OptionsFirst.vue.txt" => include_str!(
            "../../../../../tests/_fixtures/differential/linter/macro-order-config-6097/OptionsFirst.vue.txt"
        ),
        "EmitsFirst.vue.txt" => include_str!(
            "../../../../../tests/_fixtures/differential/linter/macro-order-config-6097/EmitsFirst.vue.txt"
        ),
        other => panic!("unknown authored macro input: {other}"),
    }
}

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(false)
        .with_help_level(HelpLevel::Full)
        .with_enabled_rules(Some(vec![RULE.into()]))
}

fn configured(case: &Case) -> Linter {
    let options: ConfigLintRuleOptions = serde_json::from_value(match &case.options {
        Some(options) => json!({ "script/define-macros-order": options }),
        None => json!({}),
    })
    .unwrap();
    linter().with_config_rule_options(&options)
}

fn packet(result: &LintResult) -> Value {
    json!({
        "filename": result.filename.as_str(),
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|diagnostic| json!({
            "rule_name": diagnostic.rule_name,
            "severity": diagnostic.severity,
            "message": diagnostic.message.as_str(),
            "start": diagnostic.start,
            "end": diagnostic.end,
            "help": diagnostic.help.as_ref().map(|help| help.as_str()),
            "labels": diagnostic.labels.iter().map(|label| json!({
                "message": label.message.as_str(),
                "start": label.start,
                "end": label.end,
            })).collect::<Vec<_>>(),
            "fix": diagnostic.fix,
        })).collect::<Vec<_>>(),
    })
}

// Existing Vize diagnostics quote macro names; the pinned ESLint messages do not.
fn native_message(case: &Case) -> &'static str {
    let (reference, native) = match case.id.as_str() {
        "options-first-default" => (
            "defineProps should be placed at the top of `<script setup>` (after any potential import statements or type definitions).",
            "`defineProps` should be placed at the top of `<script setup>` (after any potential import statements or type definitions).",
        ),
        "emits-first-default" | "emits-first-default-object" => (
            "defineProps should be above defineEmits.",
            "`defineProps` should be above `defineEmits`.",
        ),
        other => panic!("unknown macro finding: {other}"),
    };
    assert_eq!(case.reference.messages[0].message.as_str(), reference);
    native
}

fn expected(case: &Case) -> Value {
    let source = source(case);
    let diagnostics = match &case.statement {
        None => {
            assert!(case.reference.messages.is_empty());
            Vec::new()
        }
        Some(statement) => {
            assert_eq!(case.reference.messages.len(), 1);
            let start = source.find(statement.as_str()).unwrap();
            assert_eq!(source.matches(statement.as_str()).count(), 1);
            let end = start + statement.len();
            vec![json!({
                "rule_name": RULE,
                "severity": "warning",
                "message": native_message(case),
                "start": start,
                "end": end,
                "help": HELP,
                "labels": [{ "message": "compiler macro out of order", "start": start, "end": end }],
                "fix": null,
            })]
        }
    };
    json!({ "filename": FILENAME, "error_count": 0, "warning_count": diagnostics.len(), "diagnostics": diagnostics })
}

#[test]
fn typed_macro_options_preserve_complete_original_packets_and_requery() {
    let corpus: Corpus = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(corpus.cases.len(), 6);
    for case in &corpus.cases {
        let source = source(case);
        let configured = configured(case);
        let expected = expected(case);
        for _ in 0..2 {
            assert_eq!(
                packet(&configured.lint_sfc(source, FILENAME)),
                expected,
                "{}",
                case.id
            );
        }
        let direct = match &case.options {
            Some(options) => linter().with_define_macros_order(options.clone()),
            None => linter(),
        };
        assert_eq!(
            packet(&direct.lint_sfc(source, FILENAME)),
            expected,
            "{}",
            case.id
        );
        if matches!(
            case.id.as_str(),
            "options-first-configured" | "emits-first-reversed"
        ) {
            assert_ne!(
                packet(&linter().lint_sfc(source, FILENAME)),
                expected,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn typed_macro_options_override_direct_policy_and_empty_object_resets_default() {
    let corpus: Corpus = serde_json::from_str(CORPUS).unwrap();
    let baseline = &corpus.cases[0];
    let custom = &corpus.cases[1];
    assert_eq!(custom.id, "options-first-configured");
    let source = source(custom);
    let json: ConfigLintRuleOptions =
        serde_json::from_value(json!({ "script/define-macros-order": custom.options })).unwrap();
    let linter = linter()
        .with_define_macros_order(DefineMacrosOrderOptions::default())
        .with_config_rule_options(&json);
    assert_eq!(packet(&linter.lint_sfc(source, FILENAME)), expected(custom));

    let reset: ConfigLintRuleOptions =
        serde_json::from_value(json!({ "script/define-macros-order": {} })).unwrap();
    let linter = configured(custom).with_config_rule_options(&reset);
    assert_eq!(
        packet(&linter.lint_sfc(source, FILENAME)),
        expected(baseline)
    );

    let absent = ConfigLintRuleOptions::default();
    let linter = configured(custom).with_config_rule_options(&absent);
    assert_eq!(packet(&linter.lint_sfc(source, FILENAME)), expected(custom));
}

#[test]
fn typed_macro_options_do_not_enable_an_unselected_rule() {
    let corpus: Corpus = serde_json::from_str(CORPUS).unwrap();
    let case = &corpus.cases[0];
    let options: ConfigLintRuleOptions =
        serde_json::from_value(json!({ "script/define-macros-order": {} })).unwrap();
    for enabled in [Vec::new(), vec!["vue/no-deprecated-slot-attribute".into()]] {
        let linter = Linter::with_preset(LintPreset::Essential)
            .with_type_aware_lint(false)
            .with_enabled_rules(Some(enabled))
            .with_config_rule_options(&options);
        assert_eq!(
            packet(&linter.lint_sfc(source(case), FILENAME)),
            json!({
                "filename": FILENAME, "error_count": 0, "warning_count": 0, "diagnostics": [],
            })
        );
    }
}
