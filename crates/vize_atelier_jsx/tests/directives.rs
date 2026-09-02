//! Lowering of Vue directive (`v-x`) attribute syntax in JSX.

mod common;

use common::{find_directive, lower_one, root_element, simple_content};
use vize_s0::Allocator;

#[test]
fn v_model_directive() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <input v-model={value}/>;");
    let element = root_element(&root);
    let directive = find_directive(element, "model").expect("v-model directive");
    assert!(directive.arg.is_none());
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "value");
}

#[test]
fn v_show_directive() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-show={visible}/>;");
    let directive = find_directive(root_element(&root), "show").expect("v-show directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "visible");
}

#[test]
fn v_html_directive() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-html={raw}/>;");
    let directive = find_directive(root_element(&root), "html").expect("v-html directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "raw");
}

#[test]
fn namespaced_v_on_directive_has_argument() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <button v-on:click={onClick}/>;");
    let directive = find_directive(root_element(&root), "on").expect("v-on directive");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "click");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "onClick");
}

#[test]
fn custom_directive_with_argument() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-focus:lazy={opts}/>;");
    let directive = find_directive(root_element(&root), "focus").expect("v-focus directive");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "lazy");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "opts");
}

#[test]
fn directive_without_value_has_no_expression() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-focus/>;");
    let directive = find_directive(root_element(&root), "focus").expect("v-focus directive");
    assert!(directive.exp.is_none());
    assert!(directive.arg.is_none());
}

#[test]
fn directive_with_string_value_is_static() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-pre=\"keep\"/>;");
    let directive = find_directive(root_element(&root), "pre").expect("v-pre directive");
    assert!(common::is_static(directive.exp.as_ref().unwrap()));
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "keep");
}

#[test]
fn directive_and_plain_attributes_coexist() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <input class=\"f\" v-model={v}/>;");
    let element = root_element(&root);
    assert!(find_directive(element, "model").is_some());
    assert_eq!(element.props.len(), 2);
    let attr = common::as_attribute(&element.props[0]);
    assert_eq!(attr.name, "class");
    assert_eq!(attr.value.as_ref().unwrap().content, "f");
}

// -- custom directive array form (#3421) -------------------------------------
//
// babel-plugin-jsx encodes a custom directive's value, argument and modifiers
// positionally in an array literal. Vize used to pass the whole array through as
// the bound value, so `[val, 'arg', ['a','b']]` reached the runtime as one
// array argument instead of three.
//
// The unpack is deliberately restricted to the shapes babel actually encodes.
// Everything else keeps the array intact, because a partial unpack would place
// what it recognizes and silently drop the rest — the exact failure mode #3421
// is about.

fn modifier_names<'a>(directive: &'a vize_relief::DirectiveNode<'a>) -> Vec<&'a str> {
    directive
        .modifiers
        .iter()
        .map(|modifier| modifier.content)
        .collect()
}

#[test]
fn custom_directive_array_unpacks_value_arg_and_modifiers() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom={[val, 'arg', ['a','b']]}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "val");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "arg");
    assert_eq!(modifier_names(directive), vec!["a", "b"]);
}

#[test]
fn custom_directive_array_of_one_is_just_the_value() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom={[val]}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "val");
    assert!(directive.arg.is_none());
    assert!(directive.modifiers.is_empty());
}

#[test]
fn custom_directive_array_takes_a_trailing_modifiers_list_without_an_arg() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom={[val, ['a']]}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "val");
    assert!(directive.arg.is_none());
    assert_eq!(modifier_names(directive), vec!["a"]);
}

#[test]
fn custom_directive_array_takes_an_arg_without_modifiers() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom={[val, 'arg']}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "val");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "arg");
    assert!(directive.modifiers.is_empty());
}

/// A non-literal value is not babel's encoding and never was: a user passing an
/// array-valued directive must keep passing it.
#[test]
fn custom_directive_non_literal_array_value_is_passed_through() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom={someArray}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.exp.as_ref().unwrap()), "someArray");
    assert!(directive.arg.is_none());
    assert!(directive.modifiers.is_empty());
}

/// An unrecognized tail keeps the whole array rather than unpacking the part it
/// understands. Placing `val` and dropping `other` would be a silent content
/// loss, which is worse than the divergence.
#[test]
fn custom_directive_unrecognized_array_shape_keeps_the_whole_array() {
    for source in [
        "const a = <a v-custom={[val, other]}/>;",
        "const a = <a v-custom={[val, 'arg', ['a'], extra]}/>;",
        "const a = <a v-custom={[val, 'arg', [notAString]]}/>;",
        "const a = <a v-custom={[...spread]}/>;",
    ] {
        let bump = Allocator::new();
        let root = lower_one(&bump, source);
        let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
        let value = simple_content(directive.exp.as_ref().unwrap());
        assert!(
            value.starts_with('['),
            "{source} should keep the array as the bound value, got {value}"
        );
        assert!(directive.arg.is_none(), "{source} should have no argument");
        assert!(
            directive.modifiers.is_empty(),
            "{source} should have no modifiers"
        );
    }
}

/// With the argument already spelled as a JSX namespace there is no positional
/// slot for the array to fill, so it stays the bound value.
#[test]
fn custom_directive_with_namespaced_arg_keeps_an_array_value() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <a v-custom:arg={[val, ['a']]}/>;");
    let directive = find_directive(root_element(&root), "custom").expect("v-custom directive");
    assert_eq!(simple_content(directive.arg.as_ref().unwrap()), "arg");
    assert!(
        simple_content(directive.exp.as_ref().unwrap()).starts_with('['),
        "the array stays the bound value"
    );
    assert!(directive.modifiers.is_empty());
}

/// Built-ins carry their own array meanings and must not be re-read as the
/// custom-directive encoding. `v-model` is destructured by its own path just
/// above this one; `v-show` and the rest take the array as their value.
#[test]
fn builtin_directives_do_not_use_the_custom_array_encoding() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div v-show={[visible, ['a']]}/>;");
    let directive = find_directive(root_element(&root), "show").expect("v-show directive");
    assert!(
        simple_content(directive.exp.as_ref().unwrap()).starts_with('['),
        "v-show keeps its array value"
    );
    assert!(directive.modifiers.is_empty());
}
