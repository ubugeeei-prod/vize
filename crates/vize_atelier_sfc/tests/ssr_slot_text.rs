//! An SFC's slot VNode branch keeps an element's text run as a scalar child.
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

#[test]
fn slot_element_text_is_scalar_through_the_public_sfc_api() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/ssr-slot-scalar-text-11610/Layout.vue.txt"
    );
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/ssr-slot-scalar-text-11610/module.expected.txt"
        )
    );
}
