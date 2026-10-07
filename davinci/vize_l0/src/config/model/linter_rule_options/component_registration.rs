use crate::String;
use serde::{Deserialize, Serialize};

/// Names registered by application plugins or Musea preview setup.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ComponentRegistrationOptions {
    pub globals: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;

    #[test]
    fn globals_layering_preserves_other_options_and_allows_an_empty_reset() {
        let mut base: ConfigLintRuleOptions = serde_json::from_str(
            r#"{"vue/require-component-registration":{"globals":["MyButton"]},"vue/no-mutating-props":{"shallowOnly":true}}"#,
        ).unwrap();
        assert_eq!(
            base.component_registration_globals().unwrap(),
            &["MyButton"]
        );
        assert!(!base.is_empty());
        let empty: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/require-component-registration":{"globals":[]}}"#)
                .unwrap();
        base.merge_from(&empty);
        assert_eq!(base.component_registration_globals().unwrap().len(), 0);
        assert!(base.no_mutating_props().unwrap().shallow_only);
        let unconfigured = ConfigLintRuleOptions::default();
        assert!(
            serde_json::to_value(&unconfigured)
                .unwrap()
                .get("vue/require-component-registration")
                .is_none()
        );
        base.merge_from(&unconfigured);
        assert!(base.component_registration_globals().is_some());
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(
                r#"{"vue/require-component-registration":{"globals":"MyButton"}}"#,
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(
                r#"{"vue/require-component-registration":{"globalz":[]}}"#,
            )
            .is_err()
        );
    }
}
