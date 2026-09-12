use crate::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_ssr,
    compile_ssr_with_template_syntax_and_experimental_options,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::Allocator;

#[test]
fn source_map_disabled_by_default_yields_none() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, "<div>{{ msg }}</div>");

    assert!(errors.is_empty(), "Errors: {errors:?}");
    assert!(result.map.is_none());
}

#[test]
fn source_map_enabled_emits_v3_document_and_keeps_code() {
    let allocator = Allocator::new();
    let source = "<div>{{ msg }}</div>";
    let (_, without_errors, without_map) = compile_ssr(&allocator, source);
    let (_, with_errors, with_map) = compile_ssr_with_template_syntax_and_experimental_options(
        &allocator,
        source,
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        SsrCompilerExperimentalOptions {
            source_map: true,
            source_map_filename: Some("Foo.vue".into()),
            ..SsrCompilerExperimentalOptions::default()
        },
    );

    assert!(without_errors.is_empty(), "Errors: {without_errors:?}");
    assert!(with_errors.is_empty(), "Errors: {with_errors:?}");
    assert_eq!(with_map.code, without_map.code);
    assert_eq!(with_map.preamble, without_map.preamble);

    let map = with_map.map.expect("source_map should attach SSR map");
    assert!(map.contains("\"version\":3"), "v3 source map: {map}");
    assert!(map.contains("\"sources\":[\"Foo.vue\"]"), "{map}");
    assert!(
        map.contains("\"sourcesContent\":[\"<div>{{ msg }}</div>\"]"),
        "{map}"
    );
    assert!(map.contains("\"mappings\":\"AAAA\""), "{map}");
}
