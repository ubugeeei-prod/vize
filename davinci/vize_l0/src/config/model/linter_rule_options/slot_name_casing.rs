use serde::{Deserialize, Serialize};

/// Casing accepted for rendered slot outlet names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotNameCasingStyle {
    #[default]
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
    #[serde(rename = "singleword")]
    Singleword,
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::SlotNameCasingStyle;

    #[test]
    fn slot_name_casing_is_a_validated_direct_style_option() {
        let mut options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/slot-name-casing":"kebab-case"}"#).unwrap();
        assert_eq!(
            options.slot_name_casing(),
            Some(SlotNameCasingStyle::KebabCase)
        );

        let overlay: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/slot-name-casing":"singleword"}"#).unwrap();
        options.merge_from(&overlay);
        assert_eq!(
            options.slot_name_casing(),
            Some(SlotNameCasingStyle::Singleword)
        );

        for invalid in [
            r#"{"vue/slot-name-casing":"PascalCase"}"#,
            r#"{"vue/slot-name-casing":false}"#,
            r#"{"vue/slot-name-casing":{}}"#,
        ] {
            assert!(serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err());
        }
    }
}
