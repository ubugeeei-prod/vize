use vize_atelier_sfc::{SfcParseOptions, compile_sfc, parse_sfc, types::SfcCompileOptions};

#[test]
fn standard_sfc_accepts_self_closing_native_tags_without_warnings() {
    let source = r#"<template><div><span class="icon" /><textarea /><p>text</p></div></template>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    insta::assert_snapshot!(result.code);
}
