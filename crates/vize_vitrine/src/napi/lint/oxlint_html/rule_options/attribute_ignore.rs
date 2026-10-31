use serde_json::Value;
use vize_l0::{String, cstr};

use super::{PatinaLintOptionsNapi, string_enum};

pub(super) fn parse_attribute_option(
    payload: &Value,
    options: &mut PatinaLintOptionsNapi,
) -> Result<(), String> {
    let (style, ignore) = if payload.is_object() {
        let value: vize_l0::config::AttributeHyphenationOptions =
            serde_json::from_value(payload.clone()).map_err(|error| cstr!("{error}"))?;
        let style = match value.style {
            vize_l0::config::HyphenationStyle::Always => "always",
            vize_l0::config::HyphenationStyle::Never => "never",
        };
        (
            style,
            Some(
                value
                    .ignore
                    .into_iter()
                    .map(|entry| entry.as_str().to_owned())
                    .collect(),
            ),
        )
    } else {
        (
            string_enum(payload, &["always", "never"], "vue/attribute-hyphenation")?,
            None,
        )
    };
    options.attribute_hyphenation = Some(style.to_owned());
    options.attribute_hyphenation_ignore = ignore;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PatinaLintOptionsNapi, parse_attribute_option};
    use serde_json::json;

    #[test]
    fn whole_policies_validate_before_mutation_and_a_string_resets_custom_ignore() {
        let mut options = PatinaLintOptionsNapi::default();
        parse_attribute_option(&json!({"style":"never","ignore":["active"]}), &mut options)
            .unwrap();
        for value in [
            json!({"style":null}),
            json!({"ignore":[""]}),
            json!({"ignore":["active","active"]}),
            json!({"ignoreTags":[]}),
        ] {
            assert!(parse_attribute_option(&value, &mut options).is_err());
            assert_eq!(options.attribute_hyphenation.as_deref(), Some("never"));
            assert_eq!(
                options.attribute_hyphenation_ignore.as_deref(),
                Some(["active".to_owned()].as_slice())
            );
        }
        parse_attribute_option(&json!("always"), &mut options).unwrap();
        assert_eq!(options.attribute_hyphenation.as_deref(), Some("always"));
        assert!(options.attribute_hyphenation_ignore.is_none());
    }
}
