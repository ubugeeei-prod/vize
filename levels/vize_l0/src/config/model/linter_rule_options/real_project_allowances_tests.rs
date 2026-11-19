use super::ConfigLintRuleOptions;

#[test]
fn allowances_validate_and_replace_by_config_layer() {
    let mut options: ConfigLintRuleOptions = serde_json::from_str(
        r#"{
            "vue/warn-custom-directive": { "ignore": ["hover-tooltip"] },
            "vue/no-bare-strings-in-template": { "allowlist": ["Ctrl+K", "/^[DRWXYmvx]$/u"] },
            "a11y/anchor-has-content": { "accessibleChildren": ["RetryImage"] },
            "a11y/label-has-for": { "controlComponents": ["CInputField"] }
        }"#,
    )
    .unwrap();

    assert_eq!(
        options.warn_custom_directive().unwrap().ignore,
        ["hover-tooltip"]
    );
    assert_eq!(
        options
            .no_bare_strings_in_template()
            .unwrap()
            .allowlist
            .as_deref()
            .unwrap(),
        ["Ctrl+K", "/^[DRWXYmvx]$/u"]
    );
    assert_eq!(
        options.anchor_has_content().unwrap().accessible_children,
        ["RetryImage"]
    );
    assert_eq!(
        options.label_has_for().unwrap().control_components,
        ["CInputField"]
    );

    let overlay: ConfigLintRuleOptions = serde_json::from_str(
        r#"{
            "vue/warn-custom-directive": { "ignore": [] },
            "vue/no-bare-strings-in-template": { "allowlist": [] },
            "a11y/anchor-has-content": { "accessibleChildren": ["AppImage"] },
            "a11y/label-has-for": { "controlComponents": ["AppInput"] }
        }"#,
    )
    .unwrap();
    options.merge_from(&overlay);

    assert!(options.warn_custom_directive().unwrap().ignore.is_empty());
    assert_eq!(
        options
            .no_bare_strings_in_template()
            .unwrap()
            .allowlist
            .as_deref(),
        Some(&[][..])
    );
    assert_eq!(
        options.anchor_has_content().unwrap().accessible_children,
        ["AppImage"]
    );
    assert_eq!(
        options.label_has_for().unwrap().control_components,
        ["AppInput"]
    );

    for invalid in [
        r#"{"vue/warn-custom-directive":{"unknown":[]}}"#,
        r#"{"vue/no-bare-strings-in-template":{"allowlist":[1]}}"#,
        r#"{"a11y/anchor-has-content":{"accessibleChildren":"RetryImage"}}"#,
        r#"{"a11y/label-has-for":{"controlComponents":[false]}}"#,
    ] {
        assert!(
            serde_json::from_str::<ConfigLintRuleOptions>(invalid).is_err(),
            "accepted invalid options: {invalid}"
        );
    }
}
