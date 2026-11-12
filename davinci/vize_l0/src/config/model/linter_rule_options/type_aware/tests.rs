use super::{
    ConstantLoopConditionMode, RestrictPlusOperandsOptions, ReturnAwaitOption, TypeAwareRuleOptions,
};
use crate::config::ConfigLintRuleOptions;

#[test]
fn reads_typescript_eslint_strict_type_checked_overrides() {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "type/no-confusing-void-expression": { "ignoreArrowShorthand": true },
        "type/no-unnecessary-condition": {
            "allowConstantLoopConditions": "only-allowed-literals"
        },
        "type/restrict-plus-operands": {
            "allowAny": false,
            "allowBoolean": false,
            "allowNullish": false,
            "allowNumberAndString": true,
            "allowRegExp": false
        },
        "type/restrict-template-expressions": {
            "allowBoolean": true,
            "allowNullish": false,
            "allowNumber": true
        },
        "type/return-await": "error-handling-correctness-only"
    }))
    .expect("typescript-eslint option shapes parse");
    let typed = options.type_aware();
    assert_eq!(
        typed
            .no_confusing_void_expression
            .map(|value| value.ignore_arrow_shorthand),
        Some(true)
    );
    assert_eq!(
        typed
            .no_unnecessary_condition
            .map(|value| value.constant_loop_mode()),
        Some(ConstantLoopConditionMode::OnlyAllowedLiterals)
    );
    assert_eq!(
        typed.restrict_plus_operands,
        Some(RestrictPlusOperandsOptions {
            allow_any: false,
            allow_boolean: false,
            allow_nullish: false,
            allow_number_and_string: true,
            allow_regexp: false,
            skip_compound_assignments: false,
        })
    );
    let template = typed
        .restrict_template_expressions
        .clone()
        .expect("template options");
    assert!(!template.allow_nullish);
    assert!(
        template.allow_any,
        "omitted keys keep typescript-eslint defaults"
    );
    assert_eq!(template.allow.len(), 1, "default allow list is kept");
    assert_eq!(
        typed.return_await,
        Some(ReturnAwaitOption::ErrorHandlingCorrectnessOnly)
    );
    assert!(!options.is_empty());
}

#[test]
fn boolean_loop_conditions_resolve_to_modes() {
    let typed: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
        "type/no-unnecessary-condition": { "allowConstantLoopConditions": true }
    }))
    .expect("boolean form parses");
    assert_eq!(
        typed
            .no_unnecessary_condition
            .map(|value| value.constant_loop_mode()),
        Some(ConstantLoopConditionMode::Always)
    );
}

#[test]
fn rejects_unknown_keys_and_modes() {
    for invalid in [
        serde_json::json!({ "type/restrict-plus-operands": { "allowStrings": true } }),
        serde_json::json!({ "type/return-await": "sometimes" }),
        serde_json::json!({
            "type/no-unnecessary-condition": { "allowConstantLoopConditions": "maybe" }
        }),
    ] {
        assert!(serde_json::from_value::<TypeAwareRuleOptions>(invalid).is_err());
    }
}

#[test]
fn merges_later_layers_per_rule() {
    let mut base: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
        "type/return-await": "always",
        "type/no-confusing-void-expression": { "ignoreVoidOperator": true }
    }))
    .expect("base");
    let overlay: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
        "type/return-await": "never"
    }))
    .expect("overlay");
    base.merge_from(&overlay);
    assert_eq!(base.return_await, Some(ReturnAwaitOption::Never));
    assert!(base.no_confusing_void_expression.is_some());
}
