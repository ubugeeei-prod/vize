//! `linter.ruleOptions` for the typescript-eslint checks, in the shape the
//! corsa rules read from `__ruleOptions`.
use serde_json::{Value, json};
use vize_l0::config::TypeAwareRuleOptions;

/// The configured option list for one corsa rule, if any.
pub(super) fn for_rule(options: &TypeAwareRuleOptions, corsa_rule: &str) -> Option<Value> {
    let value = match corsa_rule {
        "no-confusing-void-expression" => {
            serde_json::to_value(options.no_confusing_void_expression?).ok()?
        }
        "no-unnecessary-condition" => {
            let configured = options.no_unnecessary_condition?;
            let mut value = serde_json::to_value(configured).ok()?;
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "allowConstantLoopConditions".into(),
                    serde_json::to_value(configured.constant_loop_mode()).ok()?,
                );
            }
            value
        }
        "restrict-plus-operands" => serde_json::to_value(options.restrict_plus_operands?).ok()?,
        "restrict-template-expressions" => {
            serde_json::to_value(options.restrict_template_expressions.as_ref()?).ok()?
        }
        "return-await" => serde_json::to_value(options.return_await?).ok()?,
        _ => return None,
    };
    Some(json!([value]))
}

#[cfg(test)]
mod tests {
    use super::for_rule;
    use serde_json::json;
    use vize_l0::config::TypeAwareRuleOptions;

    fn configured() -> TypeAwareRuleOptions {
        serde_json::from_value(json!({
            "type/no-confusing-void-expression": { "ignoreArrowShorthand": true },
            "type/no-unnecessary-condition": { "allowConstantLoopConditions": true },
            "type/restrict-plus-operands": { "allowAny": false },
            "type/restrict-template-expressions": { "allowNullish": false },
            "type/return-await": "error-handling-correctness-only"
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
                "allowConstantLoopConditions": "always",
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
    }

    #[test]
    fn leaves_unconfigured_rules_on_their_defaults() {
        let options = TypeAwareRuleOptions::default();
        for rule in [
            "no-confusing-void-expression",
            "no-unnecessary-condition",
            "restrict-plus-operands",
            "restrict-template-expressions",
            "return-await",
            "no-deprecated",
        ] {
            assert_eq!(for_rule(&options, rule), None, "{rule}");
        }
    }
}
