use super::{HyphenationStyle, VOnEventHyphenationOptions};
use crate::config::ConfigLintRuleOptions;
use serde_json::{Value, json};

fn options(value: Value) -> ConfigLintRuleOptions {
    serde_json::from_value(json!({ "vue/v-on-event-hyphenation": value })).unwrap()
}

#[test]
fn style_strings_and_objects_preserve_the_original_style_getter() {
    for (value, style, ignore) in [
        (json!("never"), HyphenationStyle::Never, vec![]),
        (json!({}), HyphenationStyle::Always, vec![]),
        (
            json!({ "style": "never", "ignore": ["update:", "row-"] }),
            HyphenationStyle::Never,
            vec!["update:", "row-"],
        ),
    ] {
        let value = options(value);
        assert_eq!(value.v_on_event_hyphenation(), Some(style));
        let expected = VOnEventHyphenationOptions {
            style,
            ignore: ignore.into_iter().map(Into::into).collect(),
            autofix: false,
        };
        assert_eq!(value.v_on_event_hyphenation_options(), Some(expected));
    }
}

#[test]
fn ignores_require_unique_nonblank_strings_without_the_reserved_suffix() {
    for value in [
        json!({ "style": null }),
        json!({ "style": "sometimes" }),
        json!({ "ignore": null }),
        json!({ "ignore": "update:" }),
        json!({ "ignore": [0] }),
        json!({ "ignore": [""] }),
        json!({ "ignore": [" \n\t"] }),
        json!({ "ignore": ["\u{feff}"] }),
        json!({ "ignore": ["row:exit"] }),
        json!({ "ignore": ["row", "row"] }),
        json!({ "autofix": "true" }),
        json!({ "autofix": null }),
        json!({ "ignoreTags": [] }),
    ] {
        assert!(
            serde_json::from_value::<VOnEventHyphenationOptions>(value.clone()).is_err(),
            "{value}"
        );
    }
    for entries in [
        vec!["\u{0085}"],
        vec!["row", "Row"],
        vec!["row:exit\n"],
        vec![],
    ] {
        let value: VOnEventHyphenationOptions =
            serde_json::from_value(json!({ "ignore": entries })).unwrap();
        assert!(value.validate().is_ok());
    }
}

#[test]
fn typed_validation_matches_ecmascript_whitespace_including_bom_but_not_nel() {
    for entry in [
        "\t\n\u{b}\u{c}\r ",
        "\u{a0}\u{1680}",
        "\u{2000}\u{200a}",
        "\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}",
    ] {
        let value = VOnEventHyphenationOptions {
            ignore: vec![entry.into()],
            ..Default::default()
        };
        assert_eq!(value.validate(), Err("ignore entries must not be blank"));
    }
    let value = VOnEventHyphenationOptions {
        ignore: vec!["\u{85}".into()],
        ..Default::default()
    };
    assert_eq!(value.validate(), Ok(()));
}

#[test]
fn later_layers_replace_the_complete_policy_including_an_explicit_empty_reset() {
    let mut base = options(json!({ "style": "never", "ignore": ["update:"] }));
    base.merge_from(&ConfigLintRuleOptions::default());
    assert_eq!(base.v_on_event_hyphenation(), Some(HyphenationStyle::Never));
    base.merge_from(&options(json!({ "ignore": ["row"] })));
    assert_eq!(
        base.v_on_event_hyphenation_options(),
        Some(VOnEventHyphenationOptions {
            style: HyphenationStyle::Always,
            ignore: vec!["row".into()],
            autofix: false,
        })
    );
    base.merge_from(&options(json!({})));
    assert_eq!(
        base.v_on_event_hyphenation_options(),
        Some(VOnEventHyphenationOptions::default())
    );
    base.merge_from(&options(json!("never")));
    assert_eq!(
        base.v_on_event_hyphenation_options(),
        Some(VOnEventHyphenationOptions {
            style: HyphenationStyle::Never,
            ignore: vec![],
            autofix: false,
        })
    );
}

#[test]
fn autofix_is_an_opt_in_boolean_and_complete_policy_resets_keep_it_false() {
    let mut base = options(json!({ "style": "never", "autofix": true, "ignore": ["row"] }));
    base.merge_from(&ConfigLintRuleOptions::default());
    assert!(base.v_on_event_hyphenation_options().unwrap().autofix);
    for value in [
        json!({}),
        json!("never"),
        json!({ "ignore": [] }),
        json!({ "autofix": false }),
    ] {
        let mut reset = base.clone();
        reset.merge_from(&options(value));
        assert!(!reset.v_on_event_hyphenation_options().unwrap().autofix);
    }
    for value in [
        json!({ "autofix": "true" }),
        json!({ "autofix": 1 }),
        json!({ "autofix": null }),
        json!({ "autofixTypo": true }),
    ] {
        assert!(serde_json::from_value::<VOnEventHyphenationOptions>(value).is_err());
    }
}
