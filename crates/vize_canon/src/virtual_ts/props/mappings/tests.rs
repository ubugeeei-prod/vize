use super::prop_binding_identifier;

#[test]
fn authored_unicode_prop_locals_follow_the_same_oxc_identifier_authority_as_anchors() {
    let names = [
        "café",
        "caf_",
        "résumé",
        "日本語",
        "𐐀value",
        "a\u{200c}b",
        "$value",
        "_private",
        "ascii",
    ];
    assert_eq!(
        names.map(|name| prop_binding_identifier(name).to_string()),
        names.map(str::to_owned),
    );
}

#[test]
fn reserved_and_nonidentifier_prop_keys_keep_the_existing_synthetic_alias_policy() {
    assert_eq!(
        [
            "class",
            "default",
            "await",
            "data-id",
            "onUpdate:modelValue",
            "9start",
            "😀"
        ]
        .map(|name| prop_binding_identifier(name).to_string()),
        [
            "_class",
            "_default",
            "_await",
            "data_id",
            "onUpdate_modelValue",
            "_9start",
            "_"
        ],
    );
}
