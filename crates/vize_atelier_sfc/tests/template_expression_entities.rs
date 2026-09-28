#![expect(
    clippy::disallowed_macros,
    reason = "`insta::assert_snapshot!` expands to `format!`"
)]

use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

#[test]
fn template_expression_entities_are_decoded() {
    let source = r#"<template><span>{{ `&lt;${name}&gt;` }}</span><span>{{ '&amp;' }}</span><span :title="'&lt;b&gt;'">x</span></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn directive_entity_is_decoded_without_an_interpolation() {
    let source = r#"<template><span :title="'&lt;b&gt;'">x</span></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    let decoded = r#"<template><span :title="'<b>'">x</span></template>"#;
    let descriptor = parse_sfc(decoded, Default::default()).unwrap();
    let expected = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    assert_eq!(result.code, expected.code);
}
