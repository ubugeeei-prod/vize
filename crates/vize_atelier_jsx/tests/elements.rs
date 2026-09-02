//! Lowering of JSX elements: tags, kinds, nesting, self-closing.

mod common;

use common::{as_directive, as_element, lower_one, root_element, simple_content, vdom_code};
use vize_atelier_jsx::JsxLang;
use vize_relief::ElementType;
use vize_s0::Allocator;

#[test]
fn lowers_a_single_intrinsic_element() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div></div>;");
    let element = root_element(&root);
    assert_eq!(element.tag, "div");
    assert_eq!(element.tag_type, ElementType::Element);
}

#[test]
fn self_closing_element_is_flagged() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <img/>;");
    let element = root_element(&root);
    assert_eq!(element.tag, "img");
    assert!(element.is_self_closing);
}

#[test]
fn element_with_explicit_close_is_not_self_closing() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div></div>;");
    assert!(!root_element(&root).is_self_closing);
}

#[test]
fn capitalized_tag_is_a_component() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <MyComp/>;");
    let element = root_element(&root);
    assert_eq!(element.tag, "MyComp");
    assert_eq!(element.tag_type, ElementType::Component);
}

/// `<Foo.Bar.Baz/>` names a component **value**. It used to lower to the tag
/// string `"Foo.Bar.Baz"`, which the DOM backend emitted as
/// `resolveComponent("Foo.Bar.Baz")` — a lookup of a component name nobody
/// registers, so the element rendered as nothing with no diagnostic (#3421).
#[test]
fn member_expression_tag_lowers_to_a_dynamic_component() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <Foo.Bar.Baz/>;");
    let element = root_element(&root);
    assert_eq!(element.tag, "component");
    assert_eq!(element.tag_type, ElementType::Component);
    assert_eq!(element.props.len(), 1);
    let is_binding = as_directive(&element.props[0]);
    assert_eq!(is_binding.name, "bind");
    assert_eq!(simple_content(is_binding.arg.as_ref().unwrap()), "is");
    assert_eq!(
        simple_content(is_binding.exp.as_ref().unwrap()),
        "Foo.Bar.Baz"
    );
}

#[test]
fn this_member_tag_lowers_to_a_dynamic_component() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <this.Dynamic/>;");
    let element = root_element(&root);
    assert_eq!(element.tag, "component");
    assert_eq!(element.tag_type, ElementType::Component);
    assert_eq!(element.props.len(), 1);
    assert_eq!(
        simple_content(as_directive(&element.props[0]).exp.as_ref().unwrap()),
        "this.Dynamic"
    );
}

/// The whole emitted module for a member-expression tag. `resolveDynamicComponent`
/// returns a non-string argument unchanged, so this mounts exactly what
/// `@vue/babel-plugin-jsx`'s `createVNode(a.b.c, {"foo": 1}, null)` mounts.
#[test]
fn member_expression_tag_emits_resolve_dynamic_component() {
    assert_eq!(
        vdom_code("const A = () => <a.b.c foo={1}/>;", JsxLang::Jsx).as_str(),
        "export function render(_ctx, _cache) {\n  \
         return (_openBlock(), _createBlock(_resolveDynamicComponent(a.b.c), { foo: 1 }))\n}"
    );
}

/// The member expression keeps its own props, and the `is` binding it is
/// lowered through never leaks into the emitted props object.
#[test]
fn member_expression_tag_with_children_keeps_props_and_slots() {
    assert_eq!(
        vdom_code("const A = () => <a.b.c foo={1}>hi</a.b.c>;", JsxLang::Jsx).as_str(),
        "export function render(_ctx, _cache) {\n  \
         return (_openBlock(), _createBlock(_resolveDynamicComponent(a.b.c), { foo: 1 }, {\n    \
         default: _withCtx(() => [\n      \
         _createTextVNode(\"hi\")\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn nested_elements_are_lowered_recursively() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <ul><li></li><li></li></ul>;");
    let ul = root_element(&root);
    assert_eq!(ul.tag, "ul");
    assert_eq!(ul.children.len(), 2);
    for child in &ul.children {
        assert_eq!(as_element(child).tag, "li");
    }
}

#[test]
fn deeply_nested_tree_preserves_structure() {
    let bump = Allocator::new();
    let root = lower_one(
        &bump,
        "const a = <div><section><p><span/></p></section></div>;",
    );
    let div = root_element(&root);
    let section = as_element(&div.children[0]);
    let p = as_element(&section.children[0]);
    let span = as_element(&p.children[0]);
    assert_eq!(span.tag, "span");
    assert!(span.is_self_closing);
}

/// `svg:` and `math:` are the qualified spellings of the two foreign-content
/// namespaces, and stay verbatim. Every other prefix is diagnosed instead — see
/// `diagnostics.rs::unsupported_tag_namespace_is_reported` (#3421).
#[test]
fn known_namespaced_element_names_are_preserved() {
    for (source, tag) in [
        ("const a = <svg:circle/>;", "svg:circle"),
        ("const a = <math:mi/>;", "math:mi"),
    ] {
        let bump = Allocator::new();
        let root = lower_one(&bump, source);
        let element = root_element(&root);
        assert_eq!(element.tag, tag);
        // The local name starts lowercase -> intrinsic.
        assert_eq!(element.tag_type, ElementType::Element);
    }
}

#[test]
fn root_location_points_at_the_element() {
    let bump = Allocator::new();
    let src = "const a = <div></div>;";
    let root = lower_one(&bump, src);
    let element = root_element(&root);
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    assert_eq!(&src[start..end], "<div></div>");
}
