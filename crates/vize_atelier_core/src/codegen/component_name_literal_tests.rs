//! A component or a directive resolves through a JavaScript string literal
//! that spells exactly its name.

use vize_l0::String;

fn rendered(source: &str) -> String {
    let allocator = crate::Allocator::new();
    let options = crate::ParserOptions {
        is_native_tag: Some(vize_l0::is_platform_tag),
        ..Default::default()
    };
    let (mut root, errors) = crate::parse_with_options(&allocator, source, options);
    assert!(errors.is_empty(), "{source}: {errors:?}");
    crate::lane::transform(
        &allocator,
        &mut root,
        crate::TransformOptions::default(),
        None,
    );
    let result = super::generate(&root, crate::CodegenOptions::default());
    let mut output = String::from(result.preamble.as_str());
    output.push('\n');
    output.push_str(&result.code);
    output
}

fn joined(parts: &[&str]) -> String {
    let mut text = String::default();
    for part in parts {
        text.push_str(part);
    }
    text
}

/// Every source renders the module beside it.
fn assert_modules(cases: &[(&str, String)]) {
    let actual: Vec<String> = cases.iter().map(|(source, _)| rendered(source)).collect();
    let expected: Vec<String> = cases.iter().map(|(_, module)| module.clone()).collect();
    assert_eq!(actual, expected);
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

#[test]
fn a_component_resolves_by_a_string_literal_of_its_name() {
    assert_modules(&[
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
    ]);
}

#[test]
fn a_static_dynamic_component_name_is_a_string_literal() {
    assert_modules(&[
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
    ]);
}

#[test]
fn a_cached_dynamic_component_name_is_a_string_literal() {
    assert_modules(&[(
        r#"<component is='x"+f()+"' v-once />"#,
        String::from(
            "const { resolveDynamicComponent: _resolveDynamicComponent, createVNode: _createVNode, setBlockTracking: _setBlockTracking } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  return _cache[0] || (\n    _setBlockTracking(-1, true),\n    (_cache[0] = _createVNode(_resolveDynamicComponent(\"x\\\"+f()+\\\"\"))).cacheIndex = 0,\n    _setBlockTracking(1),\n    _cache[0]\n  )\n}",
        ),
    )]);
}

#[test]
fn a_conditional_dynamic_component_name_is_a_string_literal() {
    assert_modules(&[(
        r#"<component v-if="ok" is='x"+f()+"' />"#,
        String::from(
            "const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createBlock: _createBlock, createCommentVNode: _createCommentVNode } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (ok)\n    ? (_openBlock(), _createBlock(_resolveDynamicComponent(\"x\\\"+f()+\\\"\"), { key: 0 }))\n    : _createCommentVNode(\"v-if\", true)\n}",
        ),
    )]);
}

#[test]
fn a_listed_dynamic_component_name_is_a_string_literal() {
    assert_modules(&[(
        r#"<component v-for="item in list" is='x"+f()+"' />"#,
        String::from(
            "const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {\n    return (_openBlock(), _createBlock(_resolveDynamicComponent(\"x\\\"+f()+\\\"\")))\n  }), 256 /* UNKEYED_FRAGMENT */))\n}",
        ),
    )]);
}

#[test]
fn a_directive_resolves_by_a_string_literal_of_its_name() {
    let module = |variable: &str, name: &str| -> String {
        joined(&[
            "const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n  const ",
            variable,
            " = _resolveDirective(",
            name,
            ")\n  \n  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, null, 512 /* NEED_PATCH */)), [\n    [",
            variable,
            ", x]\n  ])\n}",
        ])
    };
    assert_modules(&[
        (
            r#"<div v-a"b="x"></div>"#,
            module("_directive_a34b", r#""a\"b""#),
        ),
        (
            r#"<div v-a\b="x"></div>"#,
            module("_directive_a92b", r#""a\\b""#),
        ),
    ]);
}
