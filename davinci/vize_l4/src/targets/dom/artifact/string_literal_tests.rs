//! Authored names reach the render function only as JavaScript string
//! literals that spell exactly those names.

use alloc::vec::Vec as StdVec;

use vize_l0::{Allocator, String};

use super::emit_dom_source;

fn rendered(source: &str) -> String {
    let allocator = Allocator::new();
    emit_dom_source(&allocator, source)
        .expect("the template emits")
        .assembled()
}

fn joined(parts: &[&str]) -> String {
    let mut text = String::default();
    for part in parts {
        text.push_str(part);
    }
    text
}

/// The module rendering one component resolved as `variable` by `name`.
fn resolved_component(variable: &str, name: &str) -> String {
    joined(&[
        "const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  const ",
        variable,
        " = _resolveComponent(",
        name,
        ")\n  \n  return (_openBlock(), _createBlock(",
        variable,
        "))\n}",
    ])
}

/// The module rendering one root `<component>` whose static name is `name`.
fn dynamic_component(name: &str) -> String {
    joined(&[
        "const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createBlock(_resolveDynamicComponent(",
        name,
        ")))\n}",
    ])
}

/// The module rendering one `<div>` with the custom directive `name`.
fn resolved_directive(variable: &str, name: &str) -> String {
    joined(&[
        "const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  const ",
        variable,
        " = _resolveDirective(",
        name,
        ")\n  \n  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, null, 512 /* NEED_PATCH */)), [\n    [",
        variable,
        ", x]\n  ])\n}",
    ])
}

#[test]
fn a_component_resolves_by_a_string_literal_of_its_name() {
    let cases = [
        (
            r#"<div is="vue:x&quot;+f()+&quot;"></div>"#,
            resolved_component("_component_x3443f40414334", r#""x\"+f()+\"""#),
        ),
        (
            r#"<Foo"+f()+" />"#,
            resolved_component("_component_Foo3443f40414334", r#""Foo\"+f()+\"""#),
        ),
        (
            r#"<div is="vue:my-thing"></div>"#,
            resolved_component("_component_my_thing", r#""my-thing""#),
        ),
        (
            r#"<div is="vue:my&amp;amp;thing"></div>"#,
            resolved_component("_component_my38amp59thing", r#""my&amp;thing""#),
        ),
    ];
    let actual: StdVec<String> = cases.iter().map(|(source, _)| rendered(source)).collect();
    let expected: StdVec<String> = cases.iter().map(|(_, module)| module.clone()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn a_static_dynamic_component_name_is_a_string_literal() {
    let cases = [
        (
            r#"<component is='x"+f()+"' />"#,
            dynamic_component(r#""x\"+f()+\"""#),
        ),
        (r#"<component is="x\" />"#, dynamic_component(r#""x\\""#)),
        (
            r#"<component is="a&quot;b" />"#,
            dynamic_component(r#""a\"b""#),
        ),
        (
            r#"<component is="a&amp;b" />"#,
            dynamic_component(r#""a&b""#),
        ),
        (
            r#"<component is="my-thing" />"#,
            dynamic_component(r#""my-thing""#),
        ),
        (
            r#"<div><component is='x"+f()+"' /></div>"#,
            String::from(
                "const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createElementBlock: _createElementBlock, createBlock: _createBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createElementBlock(\"div\", null, [\n    (_openBlock(), _createBlock(_resolveDynamicComponent(\"x\\\"+f()+\\\"\")))\n  ]))\n}",
            ),
        ),
    ];
    let actual: StdVec<String> = cases.iter().map(|(source, _)| rendered(source)).collect();
    let expected: StdVec<String> = cases.iter().map(|(_, module)| module.clone()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn a_directive_resolves_by_a_string_literal_of_its_name() {
    let cases = [(
        r#"<div v-a\b="x"></div>"#,
        resolved_directive("_directive_a92b", r#""a\\b""#),
    )];
    let actual: StdVec<String> = cases.iter().map(|(source, _)| rendered(source)).collect();
    let expected: StdVec<String> = cases.iter().map(|(_, module)| module.clone()).collect();
    assert_eq!(actual, expected);
}
