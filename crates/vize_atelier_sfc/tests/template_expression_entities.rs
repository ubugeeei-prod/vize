use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

#[test]
fn template_expression_entities_are_decoded() {
    let source = r#"<template><span>{{ `&lt;${name}&gt;` }}</span><span>{{ '&amp;' }}</span><span :title="'&lt;b&gt;'">x</span></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    assert!(result.code.contains("`<${_ctx.name}>`"), "{}", result.code);
    assert!(
        result.code.contains("_toDisplayString('&')"),
        "{}",
        result.code
    );
    assert!(result.code.contains("title: '<b>'"), "{}", result.code);
}
