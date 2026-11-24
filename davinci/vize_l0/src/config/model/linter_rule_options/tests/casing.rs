use super::super::{
    ComponentNameInTemplateCasingOptions, ConfigLintRuleOptions, CustomEventNameCasing,
    CustomEventNameCasingOptions, TemplateComponentNameCasing,
};

#[test]
fn deserializes_casing_options() {
    let json = r#"{
        "vue/component-name-in-template-casing": { "casing": "kebab-case" },
        "script/custom-event-name-casing": { "casing": "lowercase" }
    }"#;
    let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
    assert_eq!(
        options.component_name_in_template_casing,
        Some(ComponentNameInTemplateCasingOptions {
            casing: TemplateComponentNameCasing::KebabCase,
            ..ComponentNameInTemplateCasingOptions::default()
        })
    );
    assert_eq!(
        options.custom_event_name_casing,
        Some(CustomEventNameCasingOptions {
            casing: CustomEventNameCasing::Lowercase
        })
    );
    assert_eq!(
        options.component_name_in_template_casing(),
        Some(TemplateComponentNameCasing::KebabCase)
    );
    assert_eq!(
        options.custom_event_name_casing(),
        Some(CustomEventNameCasing::Lowercase)
    );
}

#[test]
fn casing_registration_options_default_true_and_scoped_replacement_is_complete() {
    let mut options: ConfigLintRuleOptions = serde_json::from_str(
        r#"{
        "vue/component-name-in-template-casing": { "globals": ["MyWidget"] }
    }"#,
    )
    .unwrap();
    let policy = options.component_name_in_template_casing_options().unwrap();
    assert!(policy.registered_components_only);
    assert_eq!(policy.globals, vec![crate::String::from("MyWidget")]);
    let overlay = serde_json::from_str(
        r#"{
        "vue/component-name-in-template-casing": { "registeredComponentsOnly": false }
    }"#,
    )
    .unwrap();
    options.merge_from(&overlay);
    let policy = options.component_name_in_template_casing_options().unwrap();
    assert!(!policy.registered_components_only);
    assert!(policy.globals.is_empty());
}

#[test]
fn casing_ignores_default_empty_and_keep_the_unconfigured_serialization() {
    let unconfigured: ConfigLintRuleOptions = serde_json::from_str(
        r#"{"vue/component-name-in-template-casing":{"registeredComponentsOnly":false}}"#,
    )
    .unwrap();
    let policy = unconfigured
        .component_name_in_template_casing_options()
        .unwrap();
    assert!(policy.ignores.is_empty());
    assert_eq!(
        serde_json::to_value(policy).unwrap(),
        serde_json::json!({
            "casing": "PascalCase",
            "registeredComponentsOnly": false,
            "globals": []
        })
    );

    let mut configured: ConfigLintRuleOptions = serde_json::from_str(
        r#"{"vue/component-name-in-template-casing":{"ignores":["primitive","/^Tres/"]}}"#,
    )
    .unwrap();
    let policy = configured
        .component_name_in_template_casing_options()
        .unwrap();
    assert_eq!(policy.ignores, ["primitive", "/^Tres/"]);
    assert!(policy.registered_components_only);
    assert_eq!(
        serde_json::from_value::<ComponentNameInTemplateCasingOptions>(
            serde_json::to_value(policy).unwrap()
        )
        .unwrap(),
        *policy
    );
    configured.merge_from(&unconfigured);
    let policy = configured
        .component_name_in_template_casing_options()
        .unwrap();
    assert!(policy.ignores.is_empty());
}

#[test]
fn casing_ignores_reject_wrong_types_and_duplicate_names() {
    for invalid in [
        r#"{"vue/component-name-in-template-casing":{"ignores":"primitive"}}"#,
        r#"{"vue/component-name-in-template-casing":{"ignores":[1]}}"#,
        r#"{"vue/component-name-in-template-casing":{"ignores":["primitive","primitive"]}}"#,
        r#"{"vue/component-name-in-template-casing":{"ignore":["primitive"]}}"#,
    ] {
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
            "accepted invalid option: {invalid}"
        );
    }
}
