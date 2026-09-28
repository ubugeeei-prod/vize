use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{SfcCompileOptions, compile_sfc_with_template_syntax, parse_sfc};

#[test]
fn duplicate_attributes_and_directives_reject_standard_and_strict_sfc_compilation() {
    for source in [
        r#"<template><div id="a" id="b">dup</div></template>"#,
        r#"<template><p v-if="ok" v-if="!ok">x</p></template>"#,
    ] {
        let descriptor = parse_sfc(source, Default::default()).unwrap();
        for mode in [TemplateSyntaxMode::Standard, TemplateSyntaxMode::Strict] {
            let error =
                compile_sfc_with_template_syntax(&descriptor, SfcCompileOptions::default(), mode)
                    .err()
                    .expect("duplicate template prop must be rejected");
            assert_eq!(error.code.as_deref(), Some("TEMPLATE_ERROR"));
        }
    }
}
