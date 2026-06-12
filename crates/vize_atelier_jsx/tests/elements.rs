//! Lowering of JSX elements: tags, kinds, nesting, self-closing.

mod common;

use common::{as_element, lower_one, root_element};
use vize_carton::Bump;
use vize_relief::ElementType;

#[test]
fn lowers_a_single_intrinsic_element() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div></div>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "div");
    assert_eq!(element.tag_type, ElementType::Element);
}

#[test]
fn self_closing_element_is_flagged() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <img/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "img");
    assert!(element.is_self_closing);
}

#[test]
fn element_with_explicit_close_is_not_self_closing() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div></div>;");
    assert!(!root_element(&root).is_self_closing);
}

#[test]
fn capitalized_tag_is_a_component() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <MyComp/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "MyComp");
    assert_eq!(element.tag_type, ElementType::Component);
}

#[test]
fn member_expression_tag_keeps_dotted_path() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <Foo.Bar.Baz/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "Foo.Bar.Baz");
    assert_eq!(element.tag_type, ElementType::Component);
}

#[test]
fn this_member_tag_is_a_component() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <this.Dynamic/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "this.Dynamic");
    assert_eq!(element.tag_type, ElementType::Component);
}

#[test]
fn nested_elements_are_lowered_recursively() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <ul><li></li><li></li></ul>;");
    let ul = root_element(&root);
    assert_eq!(ul.tag.as_str(), "ul");
    assert_eq!(ul.children.len(), 2);
    for child in &ul.children {
        assert_eq!(as_element(child).tag.as_str(), "li");
    }
}

#[test]
fn deeply_nested_tree_preserves_structure() {
    let bump = Bump::new();
    let root = lower_one(
        &bump,
        "const a = <div><section><p><span/></p></section></div>;",
    );
    let div = root_element(&root);
    let section = as_element(&div.children[0]);
    let p = as_element(&section.children[0]);
    let span = as_element(&p.children[0]);
    assert_eq!(span.tag.as_str(), "span");
    assert!(span.is_self_closing);
}

#[test]
fn namespaced_element_name_is_preserved() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <svg:circle/>;");
    let element = root_element(&root);
    assert_eq!(element.tag.as_str(), "svg:circle");
    // `circle` starts lowercase -> intrinsic.
    assert_eq!(element.tag_type, ElementType::Element);
}

#[test]
fn root_location_points_at_the_element() {
    let bump = Bump::new();
    let src = "const a = <div></div>;";
    let root = lower_one(&bump, src);
    let element = root_element(&root);
    let start = element.loc.start.offset as usize;
    let end = element.loc.end.offset as usize;
    assert_eq!(&src[start..end], "<div></div>");
}
