fn keyword(statement: &str) -> Option<&str> {
    statement.strip_prefix('@').and_then(|rest| {
        rest.split(|ch: char| {
            ch.is_ascii() && !ch.is_ascii_alphanumeric() && !matches!(ch, '-' | '_' | '\\')
        })
        .next()
    })
}

/// At-rules whose bodies contain selectors that require the SFC scope.
pub(super) fn should_recurse_at_rule(statement: &str) -> bool {
    let Some(keyword) = keyword(statement) else {
        return false;
    };
    [
        "container",
        "document",
        "-moz-document",
        "layer",
        "media",
        "scope",
        "supports",
    ]
    .iter()
    .any(|known| keyword.eq_ignore_ascii_case(known))
}

/// `@scope` resolves its prelude and `&` against the enclosing style rule.
pub(super) fn is_scope_at_rule(statement: &str) -> bool {
    keyword(statement).is_some_and(|keyword| keyword.eq_ignore_ascii_case("scope"))
}

#[cfg(test)]
mod tests {
    use super::{super::scope_css_for_pipeline, should_recurse_at_rule};

    #[test]
    fn grouping_names_end_at_the_keyword_boundary() {
        for source in [
            "@media(width>1px)",
            "@MEDIA (width>1px)",
            "@supports(display:grid)",
            "@container(width>1px)",
            "@scope(.root)",
            "@layer/*layer*/",
            "@document url-prefix()",
            "@-MOZ-document url-prefix()",
        ] {
            assert!(should_recurse_at_rule(source), "{source}");
        }
        for source in [
            "@mediax(width>1px)",
            "@media-unknown(width>1px)",
            "@media1(width>1px)",
            "@media_custom(width>1px)",
            "@mediaé(width>1px)",
            "@supports-custom(display:grid)",
            "@keyframes spin",
            "@font-face",
            "media(width>1px)",
        ] {
            assert!(!should_recurse_at_rule(source), "{source}");
        }
    }

    #[test]
    fn compact_grouping_rules_keep_the_complete_parent_and_scoped_selector() {
        for condition in [
            "@media(min-width:1px)",
            "@MEDIA(min-width:1px)",
            "@supports(display:grid)",
            "@container(width>1px)",
        ] {
            let mut source = vize_carton::String::from(".a{");
            source.push_str(condition);
            source.push_str("{&:hover{color:blue}}}");
            let mut expected = vize_carton::String::from(condition);
            expected.push_str("{.a[data-v-x]:hover{color:blue}}");
            assert_eq!(scope_css_for_pipeline(&source, "data-v-x"), expected);
        }
        assert_eq!(
            scope_css_for_pipeline("@mediax(x){.a{color:red}}", "data-v-x").as_str(),
            "@mediax(x){.a{color:red}}"
        );
    }
}
