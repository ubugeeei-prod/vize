use vize_atelier_dom::compile_template;
use vize_s0::Allocator;

#[test]
fn slot_outlet_does_not_create_an_unused_generic_prop_hoist() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(
        &allocator,
        r#"<section><slot name="title" class="headline">Fallback</slot></section>"#,
    );

    assert!(errors.is_empty(), "unexpected compiler errors: {errors:?}");
    assert!(
        !result.preamble.contains("_hoisted_"),
        "slot outlet codegen cannot consume generic prop hoists:\n{}",
        result.preamble
    );
    assert!(result.code.contains("_renderSlot(_ctx.$slots, \"title\""));
    assert!(result.code.contains("{ class: \"headline\" }"));
    assert!(result.code.contains("Fallback"));
}
