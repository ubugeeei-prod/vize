//! `linter.ruleOptions` for the typescript-eslint checks, in the shape the
//! corsa rules read from `__ruleOptions` (ESLint's option list).
use serde_json::{Value, json};
use vize_l0::config::TypeAwareRuleOptions;

/// The configured option list for one corsa rule, if any.
pub(super) fn for_rule(options: &TypeAwareRuleOptions, corsa_rule: &str) -> Option<Value> {
    let rule = TypeAwareRuleOptions::RULES
        .iter()
        .find(|name| name.strip_prefix("type/") == Some(corsa_rule))?;
    Some(json!([options.rule_value(rule)?]))
}

#[cfg(test)]
mod tests {
    use super::for_rule;
    use serde_json::json;
    use vize_l0::config::TypeAwareRuleOptions;

    fn configured() -> TypeAwareRuleOptions {
        serde_json::from_value(json!({
            "type/no-confusing-void-expression": { "ignoreArrowShorthand": true },
            "type/no-unnecessary-condition": {
                "allowConstantLoopConditions": "only-allowed-literals"
            },
            "type/restrict-plus-operands": { "allowAny": false },
            "type/restrict-template-expressions": { "allowNullish": false },
            "type/return-await": "error-handling-correctness-only",
            "type/no-misused-promises": { "checksVoidReturn": { "arguments": false } },
            "type/only-throw-error": { "allow": [{ "from": "lib", "name": "DOMException" }] }
        }))
        .expect("options")
    }

    #[test]
    fn passes_typescript_eslint_option_lists() {
        let options = configured();
        assert_eq!(
            for_rule(&options, "return-await"),
            Some(json!(["error-handling-correctness-only"]))
        );
        assert_eq!(
            for_rule(&options, "no-unnecessary-condition"),
            Some(json!([{
                "allowConstantLoopConditions": "only-allowed-literals",
                "checkTypePredicates": false,
                "allowRuleToRunWithoutStrictNullChecksIKnowWhatIAmDoing": false
            }]))
        );
        let plus = for_rule(&options, "restrict-plus-operands").expect("plus");
        assert_eq!(plus[0]["allowAny"], json!(false));
        assert_eq!(plus[0]["allowBoolean"], json!(true));
        assert_eq!(plus[0]["allowRegExp"], json!(true));
        let template = for_rule(&options, "restrict-template-expressions").expect("template");
        assert_eq!(template[0]["allowNullish"], json!(false));
        assert_eq!(
            template[0]["allow"],
            json!([{ "from": "lib", "name": ["Error", "URL", "URLSearchParams"] }])
        );
        assert_eq!(
            for_rule(&options, "no-confusing-void-expression"),
            Some(json!([{
                "ignoreArrowShorthand": true,
                "ignoreVoidOperator": false,
                "ignoreVoidReturningFunctions": false
            }]))
        );
        assert_eq!(
            for_rule(&options, "no-misused-promises"),
            Some(json!([{
                "checksConditionals": true,
                "checksSpreads": true,
                "checksVoidReturn": {
                    "arguments": false,
                    "attributes": true,
                    "inheritedMethods": true,
                    "properties": true,
                    "returns": true,
                    "variables": true
                }
            }]))
        );
        assert_eq!(
            for_rule(&options, "only-throw-error"),
            Some(json!([{
                "allow": [{ "from": "lib", "name": "DOMException" }],
                "allowRethrowing": true,
                "allowThrowingAny": true,
                "allowThrowingUnknown": true
            }]))
        );
    }

    #[test]
    fn leaves_unconfigured_rules_on_their_defaults() {
        let options = TypeAwareRuleOptions::default();
        for rule in TypeAwareRuleOptions::RULES {
            let bare = rule.strip_prefix("type/").expect("type/ prefix");
            assert_eq!(for_rule(&options, bare), None);
        }
        assert_eq!(for_rule(&configured(), "no-array-delete"), None);
    }
}
