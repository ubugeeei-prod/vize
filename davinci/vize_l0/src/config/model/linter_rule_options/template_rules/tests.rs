use super::super::ConfigLintRuleOptions;
use super::{
    NoDuplicateAttributesOptions, NoLoneTemplateOptions, NoMultipleTemplateRootOptions,
    NoStaticInlineStylesOptions, NoUnusedComponentsOptions, NoVHtmlOptions, TemplateRuleOptions,
};

fn parse(source: &str) -> ConfigLintRuleOptions {
    serde_json::from_str(source).unwrap()
}

#[test]
fn absent_options_keep_every_template_rule_unconfigured() {
    let options = parse("{}");
    assert!(options.is_empty());
    assert_eq!(options.template_rules(), &TemplateRuleOptions::default());
}

#[test]
fn upstream_option_names_parse_into_typed_fields() {
    let options = parse(
        r#"{
            "vue/no-duplicate-attributes": { "allowCoexistClass": false, "allowCoexistStyle": true },
            "vue/no-multiple-template-root": { "disallowComments": true },
            "vue/no-lone-template": { "ignoreAccessible": true },
            "vue/no-static-inline-styles": { "allowBinding": true },
            "vue/no-unused-components": { "ignoreWhenBindingPresent": false },
            "vue/no-v-html": { "ignorePattern": "^html" }
        }"#,
    );
    assert!(!options.is_empty());
    assert_eq!(
        options.template_rules(),
        &TemplateRuleOptions {
            no_duplicate_attributes: Some(NoDuplicateAttributesOptions {
                allow_coexist_class: Some(false),
                allow_coexist_style: Some(true),
            }),
            no_multiple_template_root: Some(NoMultipleTemplateRootOptions {
                disallow_comments: true,
            }),
            no_lone_template: Some(NoLoneTemplateOptions {
                ignore_accessible: true,
            }),
            no_static_inline_styles: Some(NoStaticInlineStylesOptions {
                allow_binding: true,
            }),
            no_unused_components: Some(NoUnusedComponentsOptions {
                ignore_when_binding_present: Some(false),
            }),
            no_v_html: Some(NoVHtmlOptions {
                ignore_pattern: Some("^html".into()),
            }),
        }
    );
}

#[test]
fn empty_objects_are_configured_with_upstream_defaults() {
    let options = parse(
        r#"{
            "vue/no-duplicate-attributes": {},
            "vue/no-multiple-template-root": {},
            "vue/no-lone-template": {},
            "vue/no-static-inline-styles": {},
            "vue/no-unused-components": {},
            "vue/no-v-html": {}
        }"#,
    );
    assert!(!options.is_empty());
    let rules = options.template_rules();
    assert_eq!(
        rules.no_duplicate_attributes,
        Some(NoDuplicateAttributesOptions::default())
    );
    assert_eq!(
        rules.no_multiple_template_root,
        Some(NoMultipleTemplateRootOptions::default())
    );
    assert_eq!(
        rules.no_lone_template,
        Some(NoLoneTemplateOptions::default())
    );
    assert_eq!(
        rules.no_static_inline_styles,
        Some(NoStaticInlineStylesOptions::default())
    );
    assert_eq!(
        rules.no_unused_components,
        Some(NoUnusedComponentsOptions::default())
    );
    assert_eq!(rules.no_v_html, Some(NoVHtmlOptions::default()));
}

#[test]
fn a_later_layer_replaces_each_authored_object_whole() {
    let mut options = parse(
        r#"{
            "vue/no-duplicate-attributes": { "allowCoexistClass": false },
            "vue/no-v-html": { "ignorePattern": "^html" }
        }"#,
    );
    options.merge_from(&parse(
        r#"{ "vue/no-duplicate-attributes": { "allowCoexistStyle": false } }"#,
    ));
    let rules = options.template_rules();
    assert_eq!(
        rules.no_duplicate_attributes,
        Some(NoDuplicateAttributesOptions {
            allow_coexist_class: None,
            allow_coexist_style: Some(false),
        })
    );
    assert_eq!(
        rules.no_v_html,
        Some(NoVHtmlOptions {
            ignore_pattern: Some("^html".into()),
        })
    );
}

#[test]
fn unknown_fields_and_wrong_types_are_rejected() {
    for invalid in [
        r#"{"vue/no-duplicate-attributes":{"allowCoexist":true}}"#,
        r#"{"vue/no-duplicate-attributes":{"allowCoexistClass":"false"}}"#,
        r#"{"vue/no-multiple-template-root":{"disallowComments":1}}"#,
        r#"{"vue/no-lone-template":{"ignoreAccessible":null}}"#,
        r#"{"vue/no-static-inline-styles":{"allowBindings":true}}"#,
        r#"{"vue/no-unused-components":{"ignoreWhenBindingPresent":"no"}}"#,
        r#"{"vue/no-v-html":{"ignorePattern":["^html"]}}"#,
        r#"{"vue/no-v-html":"^html"}"#,
    ] {
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
            "accepted invalid option: {invalid}"
        );
    }
}

#[test]
fn serialization_round_trips_authored_fields_only() {
    let options = parse(r#"{"vue/no-duplicate-attributes":{"allowCoexistStyle":false}}"#);
    let value = serde_json::to_value(options.template_rules()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({ "vue/no-duplicate-attributes": { "allowCoexistStyle": false } })
    );
}
