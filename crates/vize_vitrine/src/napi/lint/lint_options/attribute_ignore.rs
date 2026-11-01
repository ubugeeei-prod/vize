use vize_l0::config::{AttributeHyphenationOptions, HyphenationStyle};

use super::attribute_hyphenation_style;

pub(crate) fn validate_attribute_hyphenation_ignore(
    style: Option<&str>,
    ignore: &[String],
) -> Result<(), &'static str> {
    if style.is_some_and(|value| attribute_hyphenation_style(value).is_none()) {
        return Err("attributeHyphenation must be always or never when ignore is configured");
    }
    AttributeHyphenationOptions {
        style: HyphenationStyle::Always,
        ignore: ignore.iter().map(|entry| entry.as_str().into()).collect(),
        ignore_tags: Vec::new(),
    }
    .validate()
}

pub(crate) fn configure_patina_attribute_ignore(
    linter: vize_patina::Linter,
    style: Option<&str>,
    ignore: Option<&[String]>,
    ignore_tags: Option<&[String]>,
) -> vize_patina::Linter {
    match (ignore, ignore_tags) {
        (None, None) => linter,
        (ignore, ignore_tags) => linter.with_attribute_hyphenation_policy(
            style
                .and_then(attribute_hyphenation_style)
                .unwrap_or_default(),
            ignore
                .unwrap_or_default()
                .iter()
                .map(|entry| entry.as_str().into())
                .collect(),
            ignore_tags
                .unwrap_or_default()
                .iter()
                .map(|entry| entry.as_str().into())
                .collect(),
        ),
    }
}

pub(crate) fn validate_attribute_hyphenation_ignore_tags(
    style: Option<&str>,
    ignore_tags: &[String],
) -> Result<(), &'static str> {
    if style.is_some_and(|value| attribute_hyphenation_style(value).is_none()) {
        return Err("attributeHyphenation must be always or never when ignoreTags is configured");
    }
    AttributeHyphenationOptions {
        ignore_tags: ignore_tags
            .iter()
            .map(|entry| entry.as_str().into())
            .collect(),
        ..Default::default()
    }
    .validate()
}

#[cfg(test)]
mod tests {
    use super::validate_attribute_hyphenation_ignore;

    #[test]
    fn typed_native_lists_use_the_same_strict_literal_policy() {
        assert!(validate_attribute_hyphenation_ignore(None, &[]).is_ok());
        assert!(validate_attribute_hyphenation_ignore(Some("never"), &["active".into()]).is_ok());
        assert!(
            validate_attribute_hyphenation_ignore(Some("invalid"), &["active".into()]).is_err()
        );
        for ignore in [
            vec!["".into()],
            vec!["\u{feff}".into()],
            vec!["name:exit".into()],
            vec!["active".into(), "active".into()],
        ] {
            assert!(validate_attribute_hyphenation_ignore(Some("always"), &ignore).is_err());
        }
    }
}
