use serde::{Deserialize, Serialize};

/// Options for `vue/html-button-has-type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct HtmlButtonHasTypeOptions {
    pub button: bool,
    pub submit: bool,
    pub reset: bool,
}

impl Default for HtmlButtonHasTypeOptions {
    fn default() -> Self {
        Self {
            button: true,
            submit: true,
            reset: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::HtmlButtonHasTypeOptions;

    #[test]
    fn defaults_match_eslint_plugin_vue() {
        let options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/html-button-has-type":{}}"#).unwrap();
        assert_eq!(
            options.html_button_has_type(),
            Some(HtmlButtonHasTypeOptions::default())
        );
    }

    #[test]
    fn accepts_boolean_type_policies_and_replaces_the_whole_object() {
        let mut options: ConfigLintRuleOptions = serde_json::from_str(
            r#"{"vue/html-button-has-type":{"button":false,"submit":true,"reset":false}}"#,
        )
        .unwrap();
        assert_eq!(
            options.html_button_has_type(),
            Some(HtmlButtonHasTypeOptions {
                button: false,
                submit: true,
                reset: false,
            })
        );

        let overlay: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/html-button-has-type":{"button":true}}"#).unwrap();
        options.merge_from(&overlay);
        assert_eq!(
            options.html_button_has_type(),
            Some(HtmlButtonHasTypeOptions {
                button: true,
                submit: true,
                reset: true,
            })
        );
    }

    #[test]
    fn rejects_unknown_fields_and_non_boolean_values() {
        for invalid in [
            r#"{"vue/html-button-has-type":{"buttons":false}}"#,
            r#"{"vue/html-button-has-type":{"button":"false"}}"#,
        ] {
            assert!(
                serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
                "accepted invalid option: {invalid}"
            );
        }
    }
}
