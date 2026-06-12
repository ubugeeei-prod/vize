//! Lowering of JSX fragments (`<>...</>`).

mod common;

use common::{as_element, as_text, lower_one, root_element};
use vize_carton::Bump;
use vize_relief::ast::core::ElementType;

#[test]
fn top_level_fragment_lifts_children_to_root() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <><span/><b/></>;");
    assert_eq!(root.children.len(), 2);
    assert_eq!(as_element(&root.children[0]).tag.as_str(), "span");
    assert_eq!(as_element(&root.children[1]).tag.as_str(), "b");
}

#[test]
fn fragment_with_text_and_elements() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <>hi<b/></>;");
    assert_eq!(root.children.len(), 2);
    assert_eq!(as_text(&root.children[0]).content.as_str(), "hi");
    assert_eq!(as_element(&root.children[1]).tag.as_str(), "b");
}

#[test]
fn nested_fragment_becomes_fragment_component() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div><><p/></></div>;");
    let div = root_element(&root);
    let fragment = as_element(&div.children[0]);
    assert_eq!(fragment.tag.as_str(), "Fragment");
    assert_eq!(fragment.tag_type, ElementType::Component);
    assert_eq!(as_element(&fragment.children[0]).tag.as_str(), "p");
}

#[test]
fn empty_fragment_has_no_children() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <></>;");
    assert_eq!(root.children.len(), 0);
}
