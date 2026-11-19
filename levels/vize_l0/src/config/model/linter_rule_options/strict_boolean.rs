//! Allowances for the opt-in strict boolean expression rule.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct StrictBooleanExpressionsOptions {
    pub allow_string: bool,
    pub allow_number: bool,
    pub allow_nullable_object: bool,
    pub allow_nullable_boolean: bool,
    pub allow_nullable_string: bool,
    pub allow_nullable_number: bool,
    pub allow_nullable_enum: bool,
    pub allow_any: bool,
}

impl Default for StrictBooleanExpressionsOptions {
    fn default() -> Self {
        Self {
            allow_string: true,
            allow_number: true,
            allow_nullable_object: true,
            allow_nullable_boolean: false,
            allow_nullable_string: false,
            allow_nullable_number: false,
            allow_nullable_enum: false,
            allow_any: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StrictBooleanExpressionsOptions;
    use crate::config::ConfigLintRuleOptions;
    use serde_json::json;

    #[test]
    fn defaults_and_typed_options_match_the_public_contract() {
        let default = StrictBooleanExpressionsOptions::default();
        assert_eq!(
            serde_json::to_value(default).expect("defaults"),
            json!({
                "allowString": true, "allowNumber": true, "allowNullableObject": true,
                "allowNullableBoolean": false, "allowNullableString": false,
                "allowNullableNumber": false, "allowNullableEnum": false, "allowAny": false,
            })
        );
        let mut original: ConfigLintRuleOptions = serde_json::from_value(json!({
            "type/strict-boolean-expressions": { "allowString": false },
        }))
        .expect("typed options");
        let overlay: ConfigLintRuleOptions = serde_json::from_value(json!({
            "type/strict-boolean-expressions": { "allowNumber": false },
        }))
        .expect("overlay");
        original.merge_from(&overlay);
        let mut expected = default;
        expected.allow_number = false;
        assert_eq!(original.strict_boolean_expressions(), Some(expected));
        assert!(!original.is_empty());
        for invalid in [json!({ "allowAny": "yes" }), json!({ "allowStrng": false })] {
            assert!(serde_json::from_value::<StrictBooleanExpressionsOptions>(invalid).is_err());
        }
    }
}
