use serde::{Deserialize, Deserializer, Serialize, de::Error};

use super::ConfigLintRuleOptions;
use crate::String;

/// Options for `vue/multi-word-component-names`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct MultiWordComponentNamesOptions {
    #[serde(deserialize_with = "deserialize_unique_ignores")]
    pub ignores: Vec<String>,
}

impl ConfigLintRuleOptions {
    /// Configured component-name exemptions.
    pub fn multi_word_component_names(&self) -> Option<&MultiWordComponentNamesOptions> {
        self.multi_word_component_names.as_ref()
    }
}

fn deserialize_unique_ignores<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let ignores = Vec::<String>::deserialize(deserializer)?;
    for (index, name) in ignores.iter().enumerate() {
        if ignores
            .iter()
            .take(index)
            .any(|candidate| candidate == name)
        {
            return Err(D::Error::custom(
                "component names in ignores must be unique",
            ));
        }
    }
    Ok(ignores)
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::MultiWordComponentNamesOptions;

    #[test]
    fn defaults_match_eslint_plugin_vue() {
        let options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/multi-word-component-names":{}}"#).unwrap();
        assert_eq!(
            options.multi_word_component_names(),
            Some(&MultiWordComponentNamesOptions::default())
        );
    }

    #[test]
    fn accepts_unique_ignores_and_replaces_the_whole_object() {
        let mut options: ConfigLintRuleOptions = serde_json::from_str(
            r#"{"vue/multi-word-component-names":{"ignores":["Todo","Page"]}}"#,
        )
        .unwrap();
        assert_eq!(
            options.multi_word_component_names().unwrap().ignores,
            ["Todo", "Page"]
        );

        let overlay: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/multi-word-component-names":{"ignores":[]}}"#).unwrap();
        options.merge_from(&overlay);
        assert_eq!(
            options.multi_word_component_names(),
            Some(&MultiWordComponentNamesOptions::default())
        );
    }

    #[test]
    fn rejects_unknown_fields_wrong_types_and_duplicate_names() {
        for invalid in [
            r#"{"vue/multi-word-component-names":{"ignore":["Todo"]}}"#,
            r#"{"vue/multi-word-component-names":{"ignores":"Todo"}}"#,
            r#"{"vue/multi-word-component-names":{"ignores":["Todo","Todo"]}}"#,
        ] {
            assert!(
                serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
                "accepted invalid option: {invalid}"
            );
        }
    }
}
