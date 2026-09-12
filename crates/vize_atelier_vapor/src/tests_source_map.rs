use super::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
    compile_vapor_with_experimental_options,
};
use vize_carton::Allocator;

#[test]
fn source_map_disabled_by_default_yields_none() {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        "<div>{{ msg }}</div>",
        VaporCompilerOptions::default(),
    );

    assert!(result.error_messages.is_empty(), "Expected no errors");
    assert!(result.map.is_none());
}

#[test]
fn source_map_enabled_emits_v3_document_and_keeps_code() {
    let allocator = Allocator::new();
    let source = "<div>{{ msg }}</div>";
    let without_map = compile_vapor(&allocator, source, VaporCompilerOptions::default());
    let with_map = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions::default(),
        VaporCompilerExperimentalOptions {
            source_map: true,
            source_map_filename: Some("Foo.vue".into()),
            ..VaporCompilerExperimentalOptions::default()
        },
    );

    assert!(
        without_map.error_messages.is_empty(),
        "Expected no errors: {:?}",
        without_map.error_messages
    );
    assert!(
        with_map.error_messages.is_empty(),
        "Expected no errors: {:?}",
        with_map.error_messages
    );
    assert_eq!(with_map.code, without_map.code);
    assert_eq!(with_map.templates, without_map.templates);

    let map = with_map.map.expect("source_map should attach Vapor map");
    assert!(map.contains("\"version\":3"), "v3 source map: {map}");
    assert!(map.contains("\"sources\":[\"Foo.vue\"]"), "{map}");
    assert!(
        map.contains("\"sourcesContent\":[\"<div>{{ msg }}</div>\"]"),
        "{map}"
    );
    assert!(map.contains("\"mappings\":\"AAAA\""), "{map}");
}
