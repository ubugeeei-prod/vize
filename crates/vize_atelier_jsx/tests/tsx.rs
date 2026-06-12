//! Lowering of TSX-specific syntax (type annotations, generics, casts).

mod common;

use common::{lower_one_tsx, root_element, simple_content};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;
use vize_relief::ast::TemplateChildNode;
use vize_relief::ast::core::ElementType;

#[test]
fn typed_arrow_component_lowers() {
    let bump = Bump::new();
    let root = lower_one_tsx(
        &bump,
        "const App = (props: { id: number }): JSX.Element => <div id={props.id}/>;",
    );
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "div");
}

#[test]
fn generic_component_call_is_a_component() {
    let bump = Bump::new();
    let root = lower_one_tsx(&bump, "const a = <List<number> items={xs}/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "List");
    assert_eq!(element.tag_type, ElementType::Component);
}

#[test]
fn as_cast_inside_interpolation_keeps_source() {
    let bump = Bump::new();
    let root = lower_one_tsx(&bump, "const a = <p>{(x as string)}</p>;");
    let p = root_element(&root);
    match &p.children[0] {
        TemplateChildNode::Interpolation(interp) => {
            assert_eq!(simple_content(&interp.content), "(x as string)");
        }
        other => panic!("expected interpolation, got {:?}", other.node_type()),
    }
}

#[test]
fn non_null_assertion_in_attribute_binding() {
    let bump = Bump::new();
    let root = lower_one_tsx(&bump, "const a = <div ref={el!}/>;");
    let directive = common::as_directive(&root_element(&root).props[0]);
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "el!");
}

#[test]
fn tsx_type_annotation_rejected_as_plain_jsx() {
    // The same source is a hard error when parsed as plain `.jsx`.
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const App = (props: { id: number }) => <div/>;",
        JsxLang::Jsx,
    );
    assert!(out.has_errors());
}
