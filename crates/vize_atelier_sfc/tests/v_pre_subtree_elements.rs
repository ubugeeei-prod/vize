//! #13314: a v-pre subtree keeps every element plain in every target.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "whole compiler modules are compared as std strings"
)]

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

const LAYOUT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/v-pre-subtree-elements-13314/Layout.vue.txt"
);
/// The compiler differential case's pinned SSR module.
const SSR_REFERENCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/v-pre-subtree-elements-13314/module.expected.txt"
);

fn compile(source: &str, ssr: bool, vapor: bool) -> String {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                vapor,
                ..Default::default()
            },
            vapor,
            ..Default::default()
        },
    )
    .expect("compile SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.to_string()
}

#[test]
fn every_target_keeps_the_v_pre_subtree_plain() {
    assert_eq!(
        compile(LAYOUT, false, false),
        "import { createElementVNode as _createElementVNode, createVNode as _createVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment } from \"vue\"\n\nimport Child from './Child.vue'\n\n\nexport default {\n  __name: 'Layout',\n  setup(__props) {\n\n\nreturn (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(_Fragment, null, [\n    _createElementVNode(\"div\", null, [...(_cache[0] || (_cache[0] = [\n      _createElementVNode(\"div\", null, [\n        _createElementVNode(\"Child\", null, \"{{ x }}\")\n      ], -1 /* CACHED */)\n    ]))]),\n    _createElementVNode(\"Child\", null, \"{{ x }}\"),\n    _createElementVNode(\"div\", null, [...(_cache[1] || (_cache[1] = [\n      _createElementVNode(\"component\", { is: \"x\" }, null, -1 /* CACHED */),\n      _createElementVNode(\"my-comp\", null, null, -1 /* CACHED */),\n      _createElementVNode(\"slot\", null, null, -1 /* CACHED */),\n      _createElementVNode(\"template\", { \"v-if\": \"a\" }, \"x\", -1 /* CACHED */)\n    ]))]),\n    _createElementVNode(\"div\", null, [\n      _createVNode(Child)\n    ])\n  ], 64 /* STABLE_FRAGMENT */))\n}\n}\n\n}"
    );
    assert_eq!(compile(LAYOUT, true, false), SSR_REFERENCE);
    assert_eq!(
        compile(LAYOUT, false, true),
        "import { defineVaporComponent as _defineVaporComponent } from 'vue'\nimport { createComponentWithFallback as _createComponentWithFallback, setInsertionState as _setInsertionState, template as _template } from 'vue';\n\nconst t0 = _template(\"<div><div><Child>{{ x }}</Child></div></div>\")\nconst t1 = _template(\"<Child>{{ x }}</Child>\")\nconst t2 = _template(\"<div><component is=\\\"x\\\"></component><my-comp></my-comp><slot></slot><template v-if=\\\"a\\\">x</template></div>\")\nconst t3 = _template(\"<div></div>\")\n\nfunction render(_ctx, $props, $emit, $attrs, $slots) {\n  const n0 = t0()\n  const n1 = t1()\n  const n2 = t2()\n  const n4 = t3()\n  const _component_Child = _ctx.Child\n  _setInsertionState(n4)\n  const n3 = _createComponentWithFallback(_component_Child, null, null, false)\n  return [n0, n1, n2, n4]\n}\nconst __vaporRender = render\n\nimport Child from './Child.vue'\n\n\nexport default /*@__PURE__*/_defineVaporComponent({\n  __name: 'Layout',\n  render: __vaporRender,\n  setup(__props, { expose: __expose }) {\n  __expose();\n\n\nconst __returned__ = { Child }\nObject.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\nreturn __returned__\n}\n\n})"
    );
}

/// The inline render closure, from its opening line through its own `}`.
fn render_closure(code: &str) -> &str {
    let start = code
        .find("return (_ctx, _cache) => {\n")
        .expect("inline render closure");
    let end = code[start..].find("\n}\n").expect("closure end") + start + 3;
    &code[start..end]
}

/// Each closure is `@vue/compiler-sfc` 3.5.35 `compileScript({ inlineTemplate:
/// true })` output, apart from two kept spellings: a literal `<component>` keeps
/// its `is` (Vue drops it here, SSR keeps it) and a literal `<template>` passes
/// its text as a string.
#[test]
fn inline_render_closures_match_vue() {
    for (template, closure) in [
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [...(_cache[0] || (_cache[0] = [\n    _createElementVNode(\"div\", null, [\n      _createElementVNode(\"Child\", null, \"{{ x }}\")\n    ], -1 /* CACHED */)\n  ]))]))\n}\n",
        ),
        (
            "<Child v-pre>{{ x }}</Child>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"Child\", null, \"{{ x }}\"))\n}\n",
        ),
        (
            "<div v-pre><component is=\"x\"/></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [...(_cache[0] || (_cache[0] = [\n    _createElementVNode(\"component\", { is: \"x\" }, null, -1 /* CACHED */)\n  ]))]))\n}\n",
        ),
        (
            "<div v-pre><my-comp/></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [...(_cache[0] || (_cache[0] = [\n    _createElementVNode(\"my-comp\", null, null, -1 /* CACHED */)\n  ]))]))\n}\n",
        ),
        (
            "<div v-pre><slot/></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [...(_cache[0] || (_cache[0] = [\n    _createElementVNode(\"slot\", null, null, -1 /* CACHED */)\n  ]))]))\n}\n",
        ),
        (
            "<div v-pre><template v-if=\"a\">x</template></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [...(_cache[0] || (_cache[0] = [\n    _createElementVNode(\"template\", { \"v-if\": \"a\" }, \"x\", -1 /* CACHED */)\n  ]))]))\n}\n",
        ),
        (
            "<div><Child/></div>",
            "return (_ctx, _cache) => {\n  return (_openBlock(), _createElementBlock(\"div\", null, [\n    _createVNode(Child)\n  ]))\n}\n",
        ),
    ] {
        let source = format!(
            "<script setup>\nimport Child from './Child.vue'\n</script>\n\n<template>\n  {template}\n</template>\n"
        );
        let code = compile(&source, false, false);
        assert_eq!(render_closure(&code), closure, "{template}");
    }
}
