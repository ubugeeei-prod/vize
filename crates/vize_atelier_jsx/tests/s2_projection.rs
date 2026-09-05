//! P2-16 JSX-to-S2 projection witnesses.

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_s0::Allocator;
use vize_s2::op::{BindingOp, DynamicName, Op};

#[test]
fn dynamic_bind_props_project_to_s2_bindings() {
    let allocator = Allocator::new();
    let source = "const App = () => <div id={name} {...attrs} disabled />";
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    assert!(!lowered.has_errors(), "{:?}", lowered.diagnostics);
    let root = lowered.roots.first().expect("one JSX root");

    let s2 = root.s2.as_ref().expect("dynamic bindings are admitted");
    assert_eq!(s2.op_count, 3);
    let Op::Element(element) = &s2.root.ops[0] else {
        panic!("root is an element");
    };
    assert_eq!(element.attributes.len(), 1);
    assert_eq!(element.attributes[0].name, "disabled");
    assert_eq!(element.bindings.len(), 2);

    let BindingOp::Bind(id) = &element.bindings[0] else {
        panic!("first binding is ui.bind");
    };
    assert!(matches!(id.name, Some(DynamicName::Static("id"))));
    let id_value = id.value.expect("id binding has a value");
    assert_eq!(id_value.source(), "name");
    assert_eq!(id_value.span().start, source.find("name").unwrap() as u32);
    assert_eq!(id.span.start, source.find("id={name}").unwrap() as u32);

    let BindingOp::Bind(spread) = &element.bindings[1] else {
        panic!("second binding is ui.bind");
    };
    assert!(spread.name.is_none());
    let spread_value = spread.value.expect("spread binding has a value");
    assert_eq!(spread_value.source(), "attrs");
    assert_eq!(
        spread_value.span().start,
        source.find("attrs").unwrap() as u32
    );
    assert_eq!(spread.span.start, source.find("{...attrs}").unwrap() as u32);
}
