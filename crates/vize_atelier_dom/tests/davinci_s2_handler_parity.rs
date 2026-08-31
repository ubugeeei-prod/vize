//! Reduced real-project handler parity witnesses.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "untyped_arrow_component_handler_stays_direct",
        r#"<FieldKeyValues @add="(key, value) => addKeyValue(headers, key, value)" />"#,
    ),
    (
        "typed_arrow_component_handler",
        r#"<FieldKeyValues @add="(key: string, value: string) => addKeyValue(headers, key, value)" />"#,
    ),
    (
        "typed_arrow_component_handler_with_static_props",
        r#"<FieldKeyValues label="Headers" @remove="(index: number) => removeKeyValue(index, headers)" />"#,
    ),
];

#[test]
fn s2_real_project_handler_parity_matches_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
