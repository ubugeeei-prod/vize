use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ContentDirectivesOptions {
    pub content_directives: Vec<crate::String>,
}

#[cfg(test)]
mod tests {
    use crate::config::ConfigLintRuleOptions;

    #[test]
    fn typed_directive_lists_replace_and_can_reset_scoped_options() {
        let mut root: ConfigLintRuleOptions = serde_json::from_str(
            r#"{"html/no-empty-palpable-content":{"contentDirectives":["safe-html","sanitize"]}}"#,
        )
        .unwrap();
        assert_eq!(
            root.palpable_content_directives().unwrap(),
            ["safe-html", "sanitize"]
        );
        assert!(!root.is_empty());
        let replace = serde_json::from_str(
            r#"{"html/no-empty-palpable-content":{"contentDirectives":["dompurify-html"]}}"#,
        )
        .unwrap();
        root.merge_from(&replace);
        assert_eq!(
            root.palpable_content_directives().unwrap(),
            ["dompurify-html"]
        );
        root.merge_from(&serde_json::from_str(r#"{"html/no-empty-palpable-content":{}}"#).unwrap());
        assert!(root.palpable_content_directives().unwrap().is_empty());
    }

    #[test]
    fn invalid_options_are_rejected_without_changing_stable_rust_shape() {
        for invalid in [
            r#"{"html/no-empty-palpable-content":{"contentDirectives":true}}"#,
            r#"{"html/no-empty-palpable-content":{"contentDirectives":[1]}}"#,
            r#"{"html/no-empty-palpable-content":{"contentDirective":["safe-html"]}}"#,
        ] {
            assert!(serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err());
        }
        let original = crate::config::LintRuleOptions {
            no_restricted_globals: None,
            no_restricted_members: None,
        };
        let options = ConfigLintRuleOptions::from_stable_options(original.clone());
        assert_eq!(options.stable_options(), &original);
        assert!(options.palpable_content_directives().is_none());
        assert!(
            !serde_json::to_value(options)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("html/no-empty-palpable-content")
        );
    }
}
