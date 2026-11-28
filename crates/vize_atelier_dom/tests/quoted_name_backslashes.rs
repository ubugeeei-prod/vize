//! A name written between quotes (a prop or event key, a dynamic prop name, a
//! directive, its argument and modifiers, a hoisted key) is spelled as a
//! JavaScript string literal on both DOM lanes. A backslash in a name is text:
//! left alone, `"e\f"` would name `e` + form feed and `"onT\u"` would not
//! parse. A double quote in a directive name is reported, as Vue reports it,
//! and no module is written.

use vize_atelier_dom::{
    DomCompilerOptions, compile_template_legacy_with_options, compile_template_with_options,
};
use vize_l0::Allocator;

const CASES: &[(&str, &str)] = &[
    (
        r#"<div c\d="1" :e\f="v" @g\h="x" v-i\j:k\l.m\n="y"></div>"#,
        r#"const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_i92j = _resolveDirective("i\\j")
  
  return _withDirectives((_openBlock(), _createElementBlock("div", {
    "c\\d": "1",
    "e\\f": _ctx.v,
    "onG\\h": _ctx.x
  }, null, 40 /* PROPS, NEED_HYDRATION */, ["e\\f", "onG\\h"])), [
    [_directive_i92j, _ctx.y, "k\\l", { "m\\n": true }]
  ])
}"#,
    ),
    (
        r#"<Comp p\q="1" :r\s="v" @t\u="x" v-model:a\b="m"><template #s\t>x</template></Comp>"#,
        r#"const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Comp = _resolveComponent("Comp")
  
  return (_openBlock(), _createBlock(_component_Comp, {
    "p\\q": "1",
    "r\\s": _ctx.v,
    "onT\\u": _ctx.x,
    "a\\b": _ctx.m,
    "onUpdate:a\\b": $event => ((_ctx.m) = $event)
  }, {
    "s\\t": _withCtx(() => [
      _createTextVNode("x")
    ]),
    _: 1 /* STABLE */
  }, 8 /* PROPS */, ["r\\s", "onT\\u", "a\\b", "onUpdate:a\\b"]))
}"#,
    ),
    (
        r#"<div><span m\n="2"></span><input @keyup.a\b="x"></div>"#,
        r#"const { withKeys: _withKeys, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, [
    _createElementVNode("span", { "m\\n": "2" }),
    _createElementVNode("input", {
      onKeyup: _withKeys(_ctx.x, ["a\\b"])
    }, null, 40 /* PROPS, NEED_HYDRATION */, ["onKeyup"])
  ]))
}"#,
    ),
    (
        r#"<div v-for="i in l" c\d="1"></div>"#,
        r#"const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

const _hoisted_1 = { "c\\d": "1" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.l, (i) => {
    return (_openBlock(), _createElementBlock("div", { "c\\d": "1" }))
  }), 256 /* UNKEYED_FRAGMENT */))
}"#,
    ),
    (
        r#"<div \a="1" data-b="2" :c-d="v"></div>"#,
        r#"const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", {
    "\\a": "1",
    "data-b": "2",
    "c-d": _ctx.v
  }, null, 8 /* PROPS */, ["c-d"]))
}"#,
    ),
    (
        r#"<div v-pre><span \a="{{ x }}"></span></div>"#,
        r#"const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, [
    _createElementVNode("span", { "\\a": "{{ x }}" })
  ]))
}"#,
    ),
];

fn options() -> DomCompilerOptions {
    DomCompilerOptions {
        hoist_static: true,
        prefix_identifiers: true,
        ..Default::default()
    }
}

#[test]
fn quoted_names_are_escaped_in_both_lanes() {
    for (source, expected) in CASES {
        let allocator = Allocator::new();
        let (_, errors, l2) = compile_template_with_options(&allocator, source, options());
        assert!(errors.is_empty(), "{source}: {errors:?}");
        let allocator = Allocator::new();
        let (_, errors, legacy) =
            compile_template_legacy_with_options(&allocator, source, options());
        assert!(errors.is_empty(), "{source}: {errors:?}");
        let module = |result: &vize_atelier_core::codegen::CodegenResult| {
            let mut module = vize_l0::String::from(result.preamble.as_str());
            module.push('\n');
            module.push_str(&result.code);
            module
        };
        assert_eq!(
            [module(&l2), module(&legacy)],
            [*expected, *expected],
            "{source}"
        );
    }
}

/// The four places a directive name can carry a double quote.
const QUOTED_DIRECTIVE_NAMES: [&str; 4] = [
    r#"<div :a"b="v"></div>"#,
    r#"<div v-x:a"b="v"></div>"#,
    r#"<div v-x.a"b="v"></div>"#,
    r#"<div @a"b="v"></div>"#,
];

const QUOTE_IN_NAME: &str = "Attribute name contains an invalid character; inferred the nearest attribute boundary and continued.";

#[test]
fn a_quote_in_a_directive_name_is_reported_in_both_lanes() {
    for source in QUOTED_DIRECTIVE_NAMES {
        for legacy in [false, true] {
            let allocator = Allocator::new();
            let (_, errors, result) = if legacy {
                compile_template_legacy_with_options(&allocator, source, options())
            } else {
                compile_template_with_options(&allocator, source, options())
            };
            let messages: Vec<_> = errors.iter().map(ToString::to_string).collect();
            assert_eq!(messages, [QUOTE_IN_NAME], "{source}");
            assert_eq!(result.code.as_str(), "", "{source}: no module is written");
        }
    }
}
