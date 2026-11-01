use super::{AttributeHyphenationConfig, AttributeHyphenationOptions, HyphenationStyle};
use crate::config::ConfigLintRuleOptions;
use serde_json::json;

fn config(value: serde_json::Value) -> ConfigLintRuleOptions {
    serde_json::from_value(json!({"vue/attribute-hyphenation": value})).unwrap()
}

#[test]
fn style_strings_keep_the_original_accessor_and_serialized_shape() {
    for (text, style) in [
        ("always", HyphenationStyle::Always),
        ("never", HyphenationStyle::Never),
    ] {
        let value = config(json!(text));
        assert_eq!(value.attribute_hyphenation(), Some(style));
        assert_eq!(value.attribute_hyphenation_ignore(), None);
        assert_eq!(
            serde_json::to_value(value).unwrap()["vue/attribute-hyphenation"],
            json!(text)
        );
    }
}

#[test]
fn objects_default_to_always_and_preserve_literal_order_case_and_boundaries() {
    let value = config(json!({"ignore": ["active", "/foo/", "FOO", "\u{85}", "\u{200b}"]}));
    assert_eq!(
        value.attribute_hyphenation(),
        Some(HyphenationStyle::Always)
    );
    let actual: Vec<_> = value
        .attribute_hyphenation_ignore()
        .unwrap()
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(actual, vec!["active", "/foo/", "FOO", "\u{85}", "\u{200b}"]);
    let empty = config(json!({}));
    assert_eq!(
        empty.attribute_hyphenation(),
        Some(HyphenationStyle::Always)
    );
    assert_eq!(
        empty.attribute_hyphenation_ignore(),
        Some(&[] as &[crate::String])
    );
}

#[test]
fn full_policy_overlay_replaces_the_custom_list_and_a_string_resets_it() {
    let mut value = config(json!({"style":"never","ignore":["active"]}));
    value.merge_from(&serde_json::from_value(json!({})).unwrap());
    assert_eq!(value.attribute_hyphenation_ignore().unwrap().len(), 1);
    value.merge_from(&config(json!({"ignore":["foo"]})));
    assert_eq!(
        value.attribute_hyphenation(),
        Some(HyphenationStyle::Always)
    );
    assert_eq!(
        value.attribute_hyphenation_ignore().unwrap()[0].as_str(),
        "foo"
    );
    value.merge_from(&config(json!("never")));
    assert_eq!(value.attribute_hyphenation(), Some(HyphenationStyle::Never));
    assert_eq!(value.attribute_hyphenation_ignore(), None);
}

#[test]
fn malformed_lists_and_unknown_options_are_refused_instead_of_widening_exemptions() {
    for value in [
        json!({"style":null}),
        json!({"style":"sometimes"}),
        json!({"ignore":null}),
        json!({"ignore":[1]}),
        json!({"ignore":[""]}),
        json!({"ignore":[" "]}),
        json!({"ignore":["\u{feff}"]}),
        json!({"ignore":["name:exit"]}),
        json!({"ignore":["active","active"]}),
    ] {
        assert!(serde_json::from_value::<AttributeHyphenationConfig>(value).is_err());
    }
    let direct = AttributeHyphenationOptions {
        ignore: vec!["".into()],
        ..Default::default()
    };
    assert!(direct.validate().is_err());
}

#[test]
fn formerly_unknown_tag_field_admits_only_literal_unique_policies() {
    // This exact former unknown-field input now belongs to the literal-tag feature.
    let value: AttributeHyphenationConfig = serde_json::from_value(json!({"ignoreTags":["MyWidget"]})).unwrap();
    assert_eq!(value.style(), HyphenationStyle::Always);
    assert_eq!(value.ignore_tags().unwrap()[0].as_str(), "MyWidget");
    for tags in [vec![], vec!["", " ", ":exit", "//", "/name", "/x/\n"]] {
        let value: AttributeHyphenationConfig = serde_json::from_value(json!({"ignoreTags":tags})).unwrap();
        assert_eq!(value.ignore_tags().unwrap().len(), tags.len());
    }
    for tags in [json!(["/^My/"]), json!(["/MyWidget/i"]), json!(["/x/y"]), json!(["MyWidget","MyWidget"]), json!([1]), json!(null)] {
        assert!(serde_json::from_value::<AttributeHyphenationConfig>(json!({"ignoreTags":tags})).is_err());
    }
    let value: AttributeHyphenationConfig = serde_json::from_value(json!({})).unwrap();
    assert_eq!(serde_json::to_value(value).unwrap(), json!({"style":"always", "ignore":[]}));
}
