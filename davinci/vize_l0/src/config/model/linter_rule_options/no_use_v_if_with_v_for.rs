use serde::{Deserialize, Serialize};

/// Options for `vue/no-use-v-if-with-v-for`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NoUseVIfWithVForOptions {
    pub allow_using_iteration_var: bool,
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::NoUseVIfWithVForOptions;

    #[test]
    fn defaults_match_eslint_plugin_vue() {
        let options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/no-use-v-if-with-v-for":{}}"#).unwrap();
        assert_eq!(
            options.no_use_v_if_with_v_for(),
            Some(NoUseVIfWithVForOptions::default())
        );
    }

    #[test]
    fn accepts_iteration_variable_policy_and_replaces_the_whole_object() {
        let mut options: ConfigLintRuleOptions = serde_json::from_str(
            r#"{"vue/no-use-v-if-with-v-for":{"allowUsingIterationVar":true}}"#,
        )
        .unwrap();
        assert_eq!(
            options.no_use_v_if_with_v_for(),
            Some(NoUseVIfWithVForOptions {
                allow_using_iteration_var: true,
            })
        );

        let overlay: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/no-use-v-if-with-v-for":{}}"#).unwrap();
        options.merge_from(&overlay);
        assert_eq!(
            options.no_use_v_if_with_v_for(),
            Some(NoUseVIfWithVForOptions::default())
        );
    }

    #[test]
    fn rejects_unknown_fields_and_non_boolean_values() {
        for invalid in [
            r#"{"vue/no-use-v-if-with-v-for":{"allowIterationVar":true}}"#,
            r#"{"vue/no-use-v-if-with-v-for":{"allowUsingIterationVar":"true"}}"#,
        ] {
            assert!(
                serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
                "accepted invalid option: {invalid}"
            );
        }
    }
}
