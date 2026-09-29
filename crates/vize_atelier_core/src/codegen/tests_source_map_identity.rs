use super::compile_with_map;

#[test]
fn source_map_static_attr_and_text_do_not_alter_generated_code() {
    // The additive invariant extended to the new anchor kinds: a template that
    // exercises static attributes, a dynamic prop key, text, and a comment must
    // still produce byte-identical `code`/`preamble` with source maps off.
    let src = r#"<div id="app" :title="t"><!--c-->hello {{ x }}</div>"#;

    let with_map = compile_with_map(src, "Foo.vue");

    let allocator = vize_l0::Allocator::new();
    let (mut root, errors) = crate::parser::parse(&allocator, src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    crate::lane::transform(
        &allocator,
        &mut root,
        crate::options::TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    let without_map = super::super::generate(
        &root,
        crate::options::CodegenOptions {
            prefix_identifiers: true,
            source_map: false,
            filename: "Foo.vue".into(),
            ..Default::default()
        },
    );

    assert_eq!(
        with_map.code.as_str(),
        without_map.code.as_str(),
        "generated code must be byte-identical regardless of source_map flag"
    );
    assert_eq!(
        with_map.preamble.as_str(),
        without_map.preamble.as_str(),
        "preamble must be byte-identical regardless of source_map flag"
    );
    assert!(without_map.map.is_none());
}
