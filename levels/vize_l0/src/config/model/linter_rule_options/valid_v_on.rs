//! Typed custom event modifiers; options alone do not enable a rule.
use super::ConfigLintRuleOptions;
use crate::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ValidVOnOptions {
    modifiers: Vec<String>,
}

impl ConfigLintRuleOptions {
    /// Custom modifier names; an explicit empty list resets inherited options.
    pub fn valid_v_on_modifiers(&self) -> Option<&[String]> {
        self.valid_v_on
            .as_ref()
            .map(|options| options.modifiers.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigLintRuleOptions;
    use serde_json::json;

    #[test]
    fn custom_modifiers_roundtrip_merge_and_reset() {
        let mut options: ConfigLintRuleOptions = serde_json::from_value(json!({
            "vue/valid-v-on": {"modifiers": ["custom"]}
        }))
        .unwrap();
        let names = options.valid_v_on_modifiers().unwrap();
        assert!(!options.is_empty());
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "custom");
        let roundtrip = serde_json::to_value(&options).unwrap();
        assert_eq!(roundtrip["vue/valid-v-on"]["modifiers"], json!(["custom"]));
        options.merge_from(&ConfigLintRuleOptions::default());
        assert_eq!(options.valid_v_on_modifiers().unwrap().len(), 1);
        options.merge_from(
            &serde_json::from_value(json!({
                "vue/valid-v-on": {"modifiers": []}
            }))
            .unwrap(),
        );
        assert!(options.valid_v_on_modifiers().unwrap().is_empty());
        assert!(
            ConfigLintRuleOptions::default()
                .valid_v_on_modifiers()
                .is_none()
        );
        let defaults: ConfigLintRuleOptions =
            serde_json::from_value(json!({"vue/valid-v-on": {}})).unwrap();
        assert!(defaults.valid_v_on_modifiers().unwrap().is_empty());
        assert!(!defaults.is_empty());
        for invalid in [
            json!({"vue/valid-v-on": {"modifiers": [13]}}),
            json!({"vue/valid-v-on": {"modifier": []}}),
            json!({"vue/valid-v-on": true}),
        ] {
            assert!(serde_json::from_value::<ConfigLintRuleOptions>(invalid).is_err());
        }
    }
}
