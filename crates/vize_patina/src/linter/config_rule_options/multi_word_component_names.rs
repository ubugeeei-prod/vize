use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

#[test]
fn config_reaches_only_the_selected_exact_component_names() {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/multi-word-component-names": { "ignores": ["Todo"] }
    }))
    .unwrap();
    let linter = Linter::with_preset(LintPreset::Essential)
        .with_enabled_rules(Some(vec!["vue/multi-word-component-names".into()]))
        .with_config_rule_options(&options);

    let allowed = linter.lint_template("<div />", "Todo.vue");
    assert_eq!(allowed.error_count, 0, "{:#?}", allowed.diagnostics);

    let boundary = linter.lint_template("<div />", "Todos.vue");
    assert_eq!(
        boundary
            .diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic.rule_name,
                diagnostic.message.as_str(),
                diagnostic.start,
                diagnostic.end,
                diagnostic.fix.is_some(),
            ))
            .collect::<Vec<_>>(),
        vec![(
            "vue/multi-word-component-names",
            "Component name \"Todos\" should be multi-word to avoid conflicts with HTML elements",
            0,
            0,
            false,
        )]
    );
}
