use serde::{Deserialize, Serialize};

/// Project-local casing for declared prop names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PropNameCasingOptions {
    pub casing: PropNameCasingStyle,
}

/// Casing accepted for a component's declared prop names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropNameCasingStyle {
    #[default]
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "snake_case")]
    SnakeCase,
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::PropNameCasingStyle;

    #[test]
    fn prop_casing_options_validate_and_replace_as_a_complete_policy() {
        let empty = ConfigLintRuleOptions::default();
        assert!(empty.is_empty());
        assert!(
            serde_json::to_value(&empty)
                .unwrap()
                .get("vue/prop-name-casing")
                .is_none()
        );
        let mut options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/prop-name-casing":{"casing":"snake_case"}}"#).unwrap();
        assert!(!options.is_empty());
        assert_eq!(
            options.prop_name_casing(),
            Some(PropNameCasingStyle::SnakeCase)
        );
        let unchanged: ConfigLintRuleOptions = serde_json::from_str("{}").unwrap();
        options.merge_from(&unchanged);
        assert_eq!(
            options.prop_name_casing(),
            Some(PropNameCasingStyle::SnakeCase)
        );
        let defaults: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/prop-name-casing":{}}"#).unwrap();
        options.merge_from(&defaults);
        assert_eq!(
            options.prop_name_casing(),
            Some(PropNameCasingStyle::CamelCase)
        );
        assert_eq!(
            serde_json::to_value(&options).unwrap()["vue/prop-name-casing"],
            serde_json::json!({ "casing": "camelCase" })
        );
        for invalid in [
            r#"{"vue/prop-name-casing":"snake_case"}"#,
            r#"{"vue/prop-name-casing":{"casing":"snake-case"}}"#,
            r#"{"vue/prop-name-casing":{"casing":false}}"#,
            r#"{"vue/prop-name-casing":{"casing":null}}"#,
            r#"{"vue/prop-name-casing":{"ignoreProps":["someProp"]}}"#,
        ] {
            assert!(serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err());
        }
    }
}
