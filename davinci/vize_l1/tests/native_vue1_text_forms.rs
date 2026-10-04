//! Original expression-only forms mirrored by the complete pinned Vue 1 oracle.

use vize_l0::Allocator;
use vize_l1::dialect::vue1::surface;
use vize_l1::{SurfaceChild, check_fidelity};

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

#[test]
fn genuine_authored_repair_keeps_original_membership_and_always_refuses_body_admission() {
    let arena = Allocator::default();
    let source = "<a>{{ before }}<a>{{ inner }}</a>{{ tail }}</a>";
    let owner = surface::parse_component_with_authored(&arena, source).unwrap();
    assert_eq!(owner.bindings().len(), 3);
    assert!(owner.errors().is_empty());
    let normal_outer = owner.children().next().unwrap();
    assert_eq!(
        owner
            .text_for(normal_outer.children().unwrap().next().unwrap())
            .unwrap_err(),
        surface::TextRefusal::RecoveredComponent
    );
    let tree = owner.authored().unwrap();
    assert_eq!(check_fidelity(tree), Ok(()));
    let authored_outer = owner.authored_children().unwrap().next().unwrap();
    assert!(core::ptr::eq(
        authored_outer.surface(),
        tree.children.first().unwrap()
    ));
    let before = authored_outer.children().unwrap().next().unwrap();
    let SurfaceChild::Interpolation(raw) = before.surface() else {
        panic!("original interpolation");
    };
    assert_eq!(
        raw.content.text.as_ptr(),
        owner.bindings()[0].raw_content().as_ptr()
    );
    assert_eq!(before.ordinal(), 0);
    assert!(core::ptr::eq(before.component(), &owner));
    assert_eq!(
        owner.text_for(before).unwrap_err(),
        surface::TextRefusal::RecoveredComponent
    );
    let authored_inner = authored_outer.children().unwrap().nth(1).unwrap();
    assert_eq!(
        owner
            .text_for(authored_inner.children().unwrap().next().unwrap())
            .unwrap_err(),
        surface::TextRefusal::RecoveredComponent
    );
    // The actual recovered normal tree has this complete inner element as an
    // independent sibling. Its clean original local occurrence remains valid.
    let normal_inner = owner.children().nth(1).unwrap();
    let view = owner
        .text_for(normal_inner.children().unwrap().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        view.expression(),
        owner.bindings()[1].syntax().unwrap().expression().unwrap()
    ));
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}
