//! Lowering of Vue directive (`v-x`) attribute syntax in JSX.

mod common;

use common::{find_directive, lower_one, root_element, simple_content};
use vize_carton::Bump;

#[test]
fn v_model_directive() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <input v-model={value}/>;");
    let element = root_element(&root);
    let directive = find_directive(element, "model").expect("v-model directive");
    assert!(directive.arg.is_none());
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "value");
}

#[test]
fn v_show_directive() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div v-show={visible}/>;");
    let directive = find_directive(root_element(&root), "show").expect("v-show directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "visible");
}

#[test]
fn v_html_directive() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div v-html={raw}/>;");
    let directive = find_directive(root_element(&root), "html").expect("v-html directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "raw");
}

#[test]
fn namespaced_v_on_directive_has_argument() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <button v-on:click={onClick}/>;");
    let directive = find_directive(root_element(&root), "on").expect("v-on directive");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "click");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "onClick");
}

#[test]
fn custom_directive_with_argument() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div v-focus:lazy={opts}/>;");
    let directive = find_directive(root_element(&root), "focus").expect("v-focus directive");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "lazy");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "opts");
}

#[test]
fn directive_without_value_has_no_expression() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div v-focus/>;");
    let directive = find_directive(root_element(&root), "focus").expect("v-focus directive");
    assert!(directive.exp.is_none());
    assert!(directive.arg.is_none());
}

#[test]
fn directive_with_string_value_is_static() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <div v-pre=\"keep\"/>;");
    let directive = find_directive(root_element(&root), "pre").expect("v-pre directive");
    assert!(common::is_static(directive.exp.as_ref().unwrap()));
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "keep");
}

#[test]
fn directive_and_plain_attributes_coexist() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <input class=\"f\" v-model={v}/>;");
    let element = root_element(&root);
    assert!(find_directive(element, "model").is_some());
    assert_eq!(element.props.len(), 2);
    let attr = common::as_attribute(&element.props[0]);
    assert_eq!(attr.name.as_str(), "class");
    assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "f");
}
