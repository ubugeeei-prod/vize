use super::super::{
    ComponentNameInTemplateCasingOptions, ConfigLintRuleOptions, CustomEventNameCasing,
    CustomEventNameCasingOptions, TemplateComponentNameCasing,
};

#[test]
fn deserializes_casing_options() {
    let json = r#"{
        "vue/component-name-in-template-casing": { "casing": "kebab-case" },
        "script/custom-event-name-casing": { "casing": "camelCase" }
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
            casing: CustomEventNameCasing::CamelCase
        })
    );
    assert_eq!(
        options.component_name_in_template_casing(),
        Some(TemplateComponentNameCasing::KebabCase)
    );
    assert_eq!(
        options.custom_event_name_casing(),
        Some(CustomEventNameCasing::CamelCase)
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
