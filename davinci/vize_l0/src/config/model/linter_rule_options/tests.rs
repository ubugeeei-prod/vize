mod casing;

use super::{
    ConfigLintRuleOptions, DefineMacrosOrderOptions, HtmlQuoteStyle, HtmlQuotesConfig,
    HtmlQuotesOptions, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
    HyphenationStyle, LintRuleOptions, MuseaDesignToken, MustacheInterpolationSpacingStyle,
    NoMutatingPropsOptions, RestrictedGlobal, RestrictedMember, SfcElementOrderGroup,
    SfcElementOrderOptions, VBindStyle, VOnStyle, VSlotNamedStyle, VSlotStyle, VSlotStyleOptions,
    VSlotStyleValue, VSlotUniformStyle,
};

#[test]
fn mustache_interpolation_spacing_validates_and_merges() {
    let mut options: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/mustache-interpolation-spacing":"never"}"#).unwrap();
    assert_eq!(
        options.mustache_interpolation_spacing(),
        Some(MustacheInterpolationSpacingStyle::Never)
    );

    let overlay: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/mustache-interpolation-spacing":"always"}"#).unwrap();
    options.merge_from(&overlay);
    assert_eq!(
        options.mustache_interpolation_spacing(),
        Some(MustacheInterpolationSpacingStyle::Always)
    );
    assert!(
        serde_json::from_str::<ConfigLintRuleOptions>(
            r#"{"vue/mustache-interpolation-spacing":"sometimes"}"#
        )
        .is_err()
    );
}

#[test]
fn html_quotes_accepts_shorthand_and_detailed_options() {
    let shorthand: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/html-quotes":"single"}"#).unwrap();
    assert_eq!(
        shorthand.html_quotes(),
        Some(HtmlQuotesConfig::Style(HtmlQuoteStyle::Single))
    );

    let mut detailed: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/html-quotes":{"style":"single","avoidEscape":true}}"#)
            .unwrap();
    assert_eq!(
        detailed.html_quotes(),
        Some(HtmlQuotesConfig::Options(HtmlQuotesOptions {
            style: HtmlQuoteStyle::Single,
            avoid_escape: true,
        }))
    );

    let defaults: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/html-quotes":{"avoidEscape":true}}"#).unwrap();
    assert_eq!(
        defaults.html_quotes(),
        Some(HtmlQuotesConfig::Options(HtmlQuotesOptions {
            style: HtmlQuoteStyle::Double,
            avoid_escape: true,
        }))
    );

    detailed.merge_from(&shorthand);
    assert_eq!(
        detailed.html_quotes(),
        Some(HtmlQuotesConfig::Style(HtmlQuoteStyle::Single))
    );

    for invalid in [
        r#"{"vue/html-quotes":"backtick"}"#,
        r#"{"vue/html-quotes":{"avoidEscape":"yes"}}"#,
        r#"{"vue/html-quotes":{"unknown":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
            "accepted invalid option: {invalid}"
        );
    }
}

#[test]
fn v_slot_style_accepts_uniform_and_position_specific_options() {
    let uniform: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-slot-style":"longform"}"#).unwrap();
    assert_eq!(
        uniform.v_slot_style(),
        Some(VSlotStyle::Uniform(VSlotUniformStyle::Longform))
    );

    let mut options: ConfigLintRuleOptions = serde_json::from_str(
        r#"{"vue/v-slot-style":{"atComponent":"shorthand","default":"v-slot","named":"longform"}}"#,
    )
    .unwrap();
    assert_eq!(
        options.v_slot_style(),
        Some(VSlotStyle::ByPosition(VSlotStyleOptions {
            at_component: VSlotStyleValue::Shorthand,
            default_slot: VSlotStyleValue::VSlot,
            named: VSlotNamedStyle::Longform,
        }))
    );

    let defaults: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-slot-style":{"named":"longform"}}"#).unwrap();
    assert_eq!(
        defaults.v_slot_style(),
        Some(VSlotStyle::ByPosition(VSlotStyleOptions {
            at_component: VSlotStyleValue::VSlot,
            default_slot: VSlotStyleValue::Shorthand,
            named: VSlotNamedStyle::Longform,
        }))
    );

    let overlay: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-slot-style":"shorthand"}"#).unwrap();
    options.merge_from(&overlay);
    assert_eq!(
        options.v_slot_style(),
        Some(VSlotStyle::Uniform(VSlotUniformStyle::Shorthand))
    );

    for invalid in [
        r#"{"vue/v-slot-style":"v-slot"}"#,
        r#"{"vue/v-slot-style":{"named":"v-slot"}}"#,
        r#"{"vue/v-slot-style":{"unknown":"longform"}}"#,
    ] {
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
            "accepted invalid option: {invalid}"
        );
    }
}

#[test]
fn define_macros_order_options_validate_and_merge() {
    let mut options: ConfigLintRuleOptions = serde_json::from_str(
        r#"{"script/define-macros-order":{"order":["defineOptions","defineProps","defineEmits","defineSlots"]}}"#,
    )
    .expect("custom macro order");
    assert_eq!(
        options.define_macros_order(),
        Some(DefineMacrosOrderOptions {
            order: vec![
                "defineOptions".into(),
                "defineProps".into(),
                "defineEmits".into(),
                "defineSlots".into(),
            ],
        })
    );

    let overlay: ConfigLintRuleOptions = serde_json::from_str(
        r#"{"script/define-macros-order":{"order":["defineEmits","defineProps"]}}"#,
    )
    .expect("overlay macro order");
    options.merge_from(&overlay);
    assert_eq!(
        options.define_macros_order().unwrap().order,
        ["defineEmits", "defineProps"]
    );
    assert!(
        serde_json::from_str::<ConfigLintRuleOptions>(
            r#"{"script/define-macros-order":{"order":true}}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<ConfigLintRuleOptions>(
            r#"{"script/define-macros-order":{"unknown":[]}}"#
        )
        .is_err()
    );
    for order in [r#"[""]"#, r#"["defineProps","defineProps"]"#] {
        let json = format!(r#"{{"script/define-macros-order":{{"order":{order}}}}}"#);
        assert!(serde_json::from_str::<ConfigLintRuleOptions>(&json).is_err());
    }
}

#[test]
fn empty_options_deserialize_to_default() {
    let options = serde_json::from_str::<LintRuleOptions>("{}").unwrap();
    assert_eq!(options, LintRuleOptions::default());
    assert!(options.is_empty());
}

#[test]
fn deserializes_restricted_globals_with_and_without_message() {
    let json = r#"{
        "script/no-restricted-globals": {
            "globals": [
                { "name": "process", "message": "Use a typed config helper." },
                { "name": "localStorage" }
            ]
        }
    }"#;
    let options = serde_json::from_str::<LintRuleOptions>(json).unwrap();
    let globals = options.no_restricted_globals.unwrap().globals;
    assert_eq!(
        globals,
        vec![
            RestrictedGlobal {
                name: "process".into(),
                message: Some("Use a typed config helper.".into()),
            },
            RestrictedGlobal {
                name: "localStorage".into(),
                message: None,
            },
        ]
    );
    assert!(options.no_restricted_members.is_none());
}

#[test]
fn deserializes_restricted_members() {
    let json = r#"{
        "script/no-restricted-members": {
            "members": [
                { "object": "window", "property": "localStorage", "message": "Use authStorage." },
                { "object": "globalThis", "property": "process" }
            ]
        }
    }"#;
    let options = serde_json::from_str::<LintRuleOptions>(json).unwrap();
    let members = options.no_restricted_members.unwrap().members;
    assert_eq!(
        members,
        vec![
            RestrictedMember {
                object: "window".into(),
                property: "localStorage".into(),
                message: Some("Use authStorage.".into()),
            },
            RestrictedMember {
                object: "globalThis".into(),
                property: "process".into(),
                message: None,
            },
        ]
    );
    assert!(options.no_restricted_globals.is_none());
}

#[test]
fn deserializes_html_self_closing_options() {
    let json = r#"{
        "vue/html-self-closing": {
            "html": {
                "void": "any",
                "normal": "never",
                "component": "any"
            },
            "svg": "any",
            "math": "any"
        }
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
    assert_eq!(
        options.html_self_closing(),
        Some(HtmlSelfClosingOptions {
            html: HtmlSelfClosingHtmlOptions {
                void_elements: HtmlSelfClosingStyle::Any,
                normal: HtmlSelfClosingStyle::Never,
                component: HtmlSelfClosingStyle::Any,
            },
            svg: HtmlSelfClosingStyle::Any,
            math: HtmlSelfClosingStyle::Any,
        })
    );
}

#[test]
fn deserializes_vue_rule_options_requested_by_misskey() {
    let json = r#"{
        "vue/no-mutating-props": { "shallowOnly": true },
        "vue/sfc-element-order": {
            "order": ["template", "script:not([setup])", "script[setup]", "style"]
        },
        "vue/v-on-event-hyphenation": "never",
        "vue/attribute-hyphenation": "never"
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();

    assert_eq!(
        options.no_mutating_props(),
        Some(NoMutatingPropsOptions { shallow_only: true })
    );
    assert_eq!(
        options.sfc_element_order(),
        Some(SfcElementOrderOptions {
            order: vec![
                SfcElementOrderGroup::Single("template".into()),
                SfcElementOrderGroup::Single("script:not([setup])".into()),
                SfcElementOrderGroup::Single("script[setup]".into()),
                SfcElementOrderGroup::Single("style".into()),
            ],
        })
    );
    assert_eq!(
        options.v_on_event_hyphenation(),
        Some(HyphenationStyle::Never)
    );
    assert_eq!(
        options.attribute_hyphenation(),
        Some(HyphenationStyle::Never)
    );
}

#[test]
fn sfc_element_order_accepts_nested_groups() {
    let json = r#"{
        "vue/sfc-element-order": {
            "order": [["script", "template"], "i18n", "style"]
        }
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();

    assert_eq!(
        options.sfc_element_order(),
        Some(SfcElementOrderOptions {
            order: vec![
                SfcElementOrderGroup::Any(vec!["script".into(), "template".into()]),
                SfcElementOrderGroup::Single("i18n".into()),
                SfcElementOrderGroup::Single("style".into()),
            ],
        })
    );
}

#[test]
fn partial_html_self_closing_options_keep_vize_defaults() {
    let json = r#"{
        "vue/html-self-closing": {
            "html": { "normal": "never" }
        }
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
    assert_eq!(
        options.html_self_closing(),
        Some(HtmlSelfClosingOptions {
            html: HtmlSelfClosingHtmlOptions {
                void_elements: HtmlSelfClosingStyle::Always,
                normal: HtmlSelfClosingStyle::Never,
                component: HtmlSelfClosingStyle::Always,
            },
            svg: HtmlSelfClosingStyle::Always,
            math: HtmlSelfClosingStyle::Always,
        })
    );
}

#[test]
fn deserializes_musea_design_tokens_with_default_tier() {
    let json = r##"{
        "musea/prefer-design-tokens": {
            "tokens": [
                { "path": "color.primary", "value": "#3b82f6" },
                { "path": "color.danger", "value": "#ef4444", "tier": "semantic" }
            ]
        }
    }"##;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
    let tokens = options.musea_prefer_design_tokens.unwrap().tokens;
    assert_eq!(
        tokens,
        vec![
            MuseaDesignToken {
                path: "color.primary".into(),
                value: "#3b82f6".into(),
                tier: "primitive".into(),
            },
            MuseaDesignToken {
                path: "color.danger".into(),
                value: "#ef4444".into(),
                tier: "semantic".into(),
            },
        ]
    );
}

#[test]
fn stable_lint_rule_options_keep_legacy_shape() {
    let json = r#"{
        "script/no-restricted-globals": {
            "globals": [{ "name": "process" }]
        },
        "vue/component-name-in-template-casing": { "casing": "kebab-case" }
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
    assert_eq!(
        options.stable_options().restricted_globals(),
        [("process".into(), None)]
    );
    assert!(options.component_name_in_template_casing().is_some());
}

#[test]
fn unknown_fields_are_rejected() {
    let json = r#"{
        "script/no-restricted-globals": {
            "globals": [{ "name": "process", "bogus": true }]
        }
    }"#;
    assert!(serde_json::from_str::<LintRuleOptions>(json).is_err());
}

#[test]
fn unknown_musea_option_fields_are_rejected() {
    let json = r##"{
        "musea/prefer-design-tokens": {
            "toknes": [
                { "path": "color.primary", "value": "#3b82f6" }
            ]
        }
    }"##;
    assert!(serde_json::from_str::<ConfigLintRuleOptions>(json).is_err());
}

#[test]
fn invalid_html_self_closing_options_are_rejected() {
    let invalid_enum = r#"{
        "vue/html-self-closing": {
            "html": { "normal": "sometimes" }
        }
    }"#;
    assert!(serde_json::from_str::<ConfigLintRuleOptions>(invalid_enum).is_err());

    let unknown_field = r#"{
        "vue/html-self-closing": {
            "html": { "normalHtml": "never" }
        }
    }"#;
    assert!(serde_json::from_str::<ConfigLintRuleOptions>(unknown_field).is_err());
}

#[test]
fn invalid_vue_rule_options_are_rejected() {
    let invalid_hyphenation = r#"{
        "vue/v-on-event-hyphenation": "sometimes"
    }"#;
    assert!(serde_json::from_str::<ConfigLintRuleOptions>(invalid_hyphenation).is_err());

    let unknown_field = r#"{
        "vue/no-mutating-props": { "shallow": true }
    }"#;
    assert!(serde_json::from_str::<ConfigLintRuleOptions>(unknown_field).is_err());
}

#[test]
fn v_on_style_options_validate_and_merge() {
    let mut options: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-on-style":"longform"}"#).unwrap();
    assert_eq!(options.v_on_style(), Some(VOnStyle::Longform));

    let overlay: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-on-style":"shorthand"}"#).unwrap();
    options.merge_from(&overlay);
    assert_eq!(options.v_on_style(), Some(VOnStyle::Shorthand));
    assert!(
        serde_json::from_str::<ConfigLintRuleOptions>(r#"{"vue/v-on-style":"sometimes"}"#).is_err()
    );
}

#[test]
fn v_bind_style_options_validate_and_merge() {
    let mut options: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-bind-style":"longform"}"#).unwrap();
    assert_eq!(options.v_bind_style(), Some(VBindStyle::Longform));

    let overlay: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"vue/v-bind-style":"shorthand"}"#).unwrap();
    options.merge_from(&overlay);
    assert_eq!(options.v_bind_style(), Some(VBindStyle::Shorthand));
    assert!(
        serde_json::from_str::<ConfigLintRuleOptions>(r#"{"vue/v-bind-style":"sometimes"}"#)
            .is_err()
    );
}

#[test]
fn props_destructuring_options_validate_and_merge() {
    use super::PropsDestructureMode;
    let mut options: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"script/define-props-destructuring": {}}"#)
            .expect("default style option");
    assert!(!options.is_empty());
    assert_eq!(
        options.define_props_destructuring(),
        Some(PropsDestructureMode::OnlyWhenAssigned)
    );
    let overlay: ConfigLintRuleOptions =
        serde_json::from_str(r#"{"script/define-props-destructuring": {"destructure":"never"}}"#)
            .expect("explicit style option");
    options.merge_from(&overlay);
    assert_eq!(
        options.define_props_destructuring(),
        Some(PropsDestructureMode::Never)
    );
    options.merge_from(&ConfigLintRuleOptions::default());
    assert_eq!(
        options.define_props_destructuring(),
        Some(PropsDestructureMode::Never)
    );
    for json in [
        r#"{"script/define-props-destructuring": {"destructure":"sometimes"}}"#,
        r#"{"script/define-props-destructuring": {"other":true}}"#,
    ] {
        assert!(serde_json::from_str::<ConfigLintRuleOptions>(json).is_err());
    }
}
