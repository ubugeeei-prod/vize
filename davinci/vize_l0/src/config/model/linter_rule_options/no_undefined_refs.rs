//! Typed project globals for template references; options alone do not enable a rule.
use super::ConfigLintRuleOptions;
use crate::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NoUndefinedRefsOptions {
    globals: Vec<String>,
}

impl ConfigLintRuleOptions {
    /// Names a build step defines for every template, such as auto-imports;
    /// an explicit empty list resets inherited options.
    pub fn undefined_ref_globals(&self) -> Option<&[String]> {
        self.no_undefined_refs
            .as_ref()
            .map(|options| options.globals.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigLintRuleOptions;
    use serde_json::json;

    #[test]
    fn globals_roundtrip_merge_and_reset() {
        let mut options: ConfigLintRuleOptions = serde_json::from_value(json!({
            "vue/no-undefined-refs": {"globals": ["formatCount", "useStore"]}
        }))
        .unwrap();
        assert_eq!(
            options.undefined_ref_globals().unwrap(),
            ["formatCount", "useStore"]
        );
        assert!(!options.is_empty());
        let roundtrip = serde_json::to_value(&options).unwrap();
        assert_eq!(
            roundtrip["vue/no-undefined-refs"]["globals"],
            json!(["formatCount", "useStore"])
        );
        options.merge_from(&ConfigLintRuleOptions::default());
        assert_eq!(options.undefined_ref_globals().unwrap().len(), 2);
        options.merge_from(
            &serde_json::from_value(json!({
                "vue/no-undefined-refs": {"globals": []}
            }))
            .unwrap(),
        );
        assert!(options.undefined_ref_globals().unwrap().is_empty());
    }

    #[test]
    fn an_unconfigured_rule_stays_absent_and_invalid_shapes_are_rejected() {
        let unconfigured = ConfigLintRuleOptions::default();
        assert!(unconfigured.undefined_ref_globals().is_none());
        assert!(unconfigured.is_empty());
        let defaults: ConfigLintRuleOptions =
            serde_json::from_value(json!({"vue/no-undefined-refs": {}})).unwrap();
        assert!(defaults.undefined_ref_globals().unwrap().is_empty());
        assert!(!defaults.is_empty());
        for invalid in [
            json!({"vue/no-undefined-refs": {"globals": "formatCount"}}),
            json!({"vue/no-undefined-refs": {"globals": [13]}}),
            json!({"vue/no-undefined-refs": {"global": []}}),
            json!({"vue/no-undefined-refs": true}),
        ] {
            assert!(serde_json::from_value::<ConfigLintRuleOptions>(invalid).is_err());
        }
    }
}
