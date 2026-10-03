//! Original expression-only forms mirrored by the complete pinned Vue 1 oracle.

use vize_l0::Allocator;
use vize_l1::dialect::vue1::surface;

#[test]
fn original_escaped_expression_forms_keep_lf_ecmascript_trim_and_module_js_profile() {
    let arena = Allocator::default();
    for (source, expression) in [
        ("{{ value }}", "value"),
        ("{{ user.name }}", "user.name"),
        ("{{ left + right }}", "left + right"),
        ("{{ ready ? yes : no }}", "ready ? yes : no"),
        ("{{ call(value, 2) }}", "call(value, 2)"),
        ("{{ [first, next] }}", "[first, next]"),
        ("{{\n value\n}}", "value"),
        ("{{\u{00a0}\u{feff}value\u{3000}}}", "value"),
    ] {
        let owner = surface::parse_component(&arena, source).unwrap();
        let binding = &owner.bindings()[0];
        let syntax = binding.syntax().unwrap();
        assert_eq!(syntax.source().text(), expression);
        assert_eq!(syntax.hole(), None);
        assert!(syntax.source_type().is_module());
        assert!(!syntax.source_type().is_typescript());
        let original = syntax.expression().unwrap();
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        assert!(core::ptr::eq(view.expression(), original));
    }
}
