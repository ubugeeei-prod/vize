//! Lowering of JSX attributes into static attributes and `v-bind` directives.

mod common;

use common::{as_attribute, as_directive, lower_one, root_element, simple_content};
use vize_carton::Bump;

#[test]
fn static_string_attribute() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div class=\"box\"/>;");
    let attr = as_attribute(&root_element(&root).props[0]);
    assert_eq!(attr.name.as_str(), "class");
    assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "box");
}

#[test]
fn boolean_attribute_has_no_value() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <input disabled/>;");
    let attr = as_attribute(&root_element(&root).props[0]);
    assert_eq!(attr.name.as_str(), "disabled");
    assert!(attr.value.is_none());
}

#[test]
fn dynamic_attribute_becomes_bind_directive() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div class={cls}/>;");
    let directive = as_directive(&root_element(&root).props[0]);
    assert_eq!(directive.name, "bind");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "class");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "cls");
}

#[test]
fn bind_argument_is_static_and_expression_is_dynamic() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div id={x}/>;");
    let directive = as_directive(&root_element(&root).props[0]);
    assert!(common::is_static(directive.arg.as_ref().unwrap()));
    assert!(!common::is_static(directive.exp.as_ref().unwrap()));
}

#[test]
fn spread_attribute_becomes_argless_bind() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div {...props}/>;");
    let directive = as_directive(&root_element(&root).props[0]);
    assert_eq!(directive.name, "bind");
    assert!(directive.arg.is_none());
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "props");
}

#[test]
fn namespaced_attribute_name_is_joined() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <use xlink:href=\"#id\"/>;");
    let attr = as_attribute(&root_element(&root).props[0]);
    assert_eq!(attr.name.as_str(), "xlink:href");
    assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "#id");
}

#[test]
fn event_handler_is_a_dynamic_bind() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <button onClick={handler}/>;");
    let directive = as_directive(&root_element(&root).props[0]);
    assert_eq!(directive.name, "bind");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "onClick");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "handler");
}

#[test]
fn multiple_attributes_preserve_order() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div id=\"x\" class={c} hidden/>;");
    let props = &root_element(&root).props;
    assert_eq!(props.len(), 3);
    assert_eq!(as_attribute(&props[0]).name.as_str(), "id");
    assert_eq!(as_directive(&props[1]).name, "bind");
    assert_eq!(as_attribute(&props[2]).name.as_str(), "hidden");
}

#[test]
fn jsx_element_as_attribute_value_is_dynamic_bind() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <Comp icon={<Icon/>}/>;");
    let directive = as_directive(&root_element(&root).props[0]);
    assert_eq!(directive.name, "bind");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "icon");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "<Icon/>");
}
