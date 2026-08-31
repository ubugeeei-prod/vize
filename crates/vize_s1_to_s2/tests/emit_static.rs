//! P2-11 installment 5: static native HTML, interpolations, mixed
//! text siblings, and static-name binds emit the same render function
//! the shipped DOM lane does.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::{emit_dom, emit_dom_source};

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[test]
fn empty_div_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\"))
}"
    );
}

#[test]
fn div_with_text_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>hello</div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"hello\"))
}"
    );
}

#[test]
fn nested_elements_match_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div><span>hello</span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, \"hello\")
  ]))
}"
    );
}

#[test]
fn emit_dom_source_agrees_with_emit_dom() {
    let allocator = Allocator::new();
    let via_source = emit_dom_source(&allocator, "<p>hi</p>").expect("emit");
    assert_eq!(assembled("<p>hi</p>"), via_source.assembled().as_str());
}

#[test]
fn empty_div_with_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div class="x"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { class: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn multiple_static_attrs_hoist_as_one_object() {
    assert_eq!(
        assembled(r#"<div id="app" class="container">static</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { id: \"app\", class: \"container\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, \"static\"))
}"
    );
}

#[test]
fn hyphenated_attr_names_are_quoted() {
    assert_eq!(
        assembled(r#"<div data-id="1"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { \"data-id\": \"1\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn boolean_attr_emits_an_empty_string_value() {
    assert_eq!(
        assembled("<div disabled></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { disabled: \"\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn nested_static_attrs_match_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div><span class="x">hello</span></div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", { class: \"x\" }, \"hello\")
  ]))
}"
    );
}

#[test]
fn a_bound_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div :class="cls"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(cls)
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn bounded_string_class_concats_drop_the_legacy_class_patch_flag() {
    assert_eq!(
        assembled(r#"<div :class="'is-' + state + '-active'"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass('is-' + state + '-active')
  }))
}"
    );
}

#[test]
fn bounded_string_style_concats_drop_the_legacy_style_patch_flag() {
    assert_eq!(
        assembled(r#"<div :style="'width:' + size + 'px'"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle('width:' + size + 'px')
  }))
}"
    );
}

#[test]
fn unparenthesized_in_conditionals_drop_legacy_class_and_style_patch_flags() {
    assert_eq!(
        assembled(r#"<div :class="'session' in row && row.session ? 'locked' : ''"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass('session' in row && row.session ? 'locked' : '')
  }))
}"
    );
    assert_eq!(
        assembled(r#"<div :style="'visible' in state ? 'display:block' : 'display:none'"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle('visible' in state ? 'display:block' : 'display:none')
  }))
}"
    );
}

#[test]
fn parenthesized_legacy_patchless_neighbors_keep_class_and_style_flags() {
    assert_eq!(
        assembled(
            r#"<div :class="('is-' + state + '-active')" :style="('width:' + size + 'px')"></div>"#,
        ),
        "\
const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(('is-' + state + '-active')),
    style: _normalizeStyle(('width:' + size + 'px'))
  }, null, 6 /* CLASS, STYLE */))
}"
    );
}

#[test]
fn class_plus_this_style_objects_skip_the_legacy_style_normalizer() {
    assert_eq!(
        assembled(r#"<div :class="c" :style="{ height: this.h }"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(c),
    style: { height: this.h }
  }, null, 6 /* CLASS, STYLE */))
}"
    );
}

#[test]
fn dynamic_style_objects_keep_the_legacy_style_normalizer() {
    assert_eq!(
        assembled(r#"<div :style="{ height: this.h, width: w }"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle({ height: this.h, width: w })
  }, null, 4 /* STYLE */))
}"
    );
}

#[test]
fn full_props_patch_flags_drop_class_and_style_bits() {
    assert_eq!(
        assembled(r#"<div :[foo]="bar" :class="c" :style="s"></div>"#),
        "\
const { normalizeProps: _normalizeProps, normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({
    [_ctx.foo || \"\"]: bar,
    class: _normalizeClass(c),
    style: _normalizeStyle(s)
  }), null, 16 /* FULL_PROPS */))
}"
    );
    assert_eq!(
        assembled(r#"<div :class="c" @[name]="handler"></div>"#),
        "\
const { normalizeClass: _normalizeClass, toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(c),
    [_toHandlerKey(_ctx.name)]: handler
  }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_bound_id_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div :id="foo"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { id: foo }, null, 8 /* PROPS */, [\"id\"]))
}"
    );
}

#[test]
fn ts_wrapped_static_binds_keep_the_legacy_props_patch_flag() {
    assert_eq!(
        assembled(r#"<div :id="'x' as const"></div><Foo :id="'x' as const" />"#),
        concat!(
            "\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")
",
            "  \n",
            "  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createElementVNode(\"div\", { id: 'x' as const }, null, 8 /* PROPS */, [\"id\"]),
    _createVNode(_component_Foo, { id: 'x' as const }, null, 8 /* PROPS */, [\"id\"])
  ], 64 /* STABLE_FRAGMENT */))
}"
        )
    );
}

#[test]
fn simple_interpolation_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("{{ msg }}"),
        "\
const { toDisplayString: _toDisplayString } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _toDisplayString(msg)
}"
    );
}

#[test]
fn root_fragment_compound_text_drops_dynamic_gap_like_the_shipped_snapshot() {
    assert_eq!(
        assembled("x {{ a }} {{ b }}"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createTextVNode(\"x \"),
    _toDisplayString(a),
    _toDisplayString(b)
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
    assert_eq!(
        assembled("{{ a }} {{ b }}"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _toDisplayString(a),
    _toDisplayString(b)
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
}

#[test]
fn comment_bounded_dynamic_whitespace_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>{{ a }} <!--c--></div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(a), 1 /* TEXT */))
}"
    );
    assert_eq!(
        assembled("<div><!--c--> {{ a }} <!--d--></div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(a), 1 /* TEXT */))
}"
    );
    assert_eq!(
        assembled("<div>{{ a }} <!--c--><span></span></div>"),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(_toDisplayString(a) + \" \", 1 /* TEXT */),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn interpolation_in_element_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>{{ msg }}</div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(msg), 1 /* TEXT */))
}"
    );
}

#[test]
fn mixed_text_and_interpolation_compiles_from_text_facts() {
    assert_eq!(
        assembled("<div>hello {{ msg }}</div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"hello \" + _toDisplayString(msg), 1 /* TEXT */))
}"
    );
}

#[test]
fn hoisted_static_props_omit_the_text_patch_flag() {
    assert_eq!(
        assembled(r#"<div class="x">{{ msg }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { class: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, _toDisplayString(msg)))
}"
    );
}

#[test]
fn nested_interpolation_keeps_the_text_patch_flag() {
    assert_eq!(
        assembled("<div><span>{{ msg }}</span></div>"),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, _toDisplayString(msg), 1 /* TEXT */)
  ]))
}"
    );
}

#[test]
fn a_trailing_root_newline_does_not_steal_compound_child_ids() {
    assert_eq!(
        assembled("<div>Hi {{ name }}</div>\n"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"Hi \" + _toDisplayString(name), 1 /* TEXT */))
}"
    );
}

#[test]
fn mixed_element_and_interpolation_siblings_use_create_text_vnode() {
    assert_eq!(
        assembled("<div>{{ msg }}<span></span></div>"),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(_toDisplayString(msg), 1 /* TEXT */),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn mixed_static_text_and_element_siblings_use_create_text_vnode() {
    assert_eq!(
        assembled("<div>hello<span></span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(\"hello\"),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn a_single_space_between_elements_is_create_text_vnode_with_no_args() {
    assert_eq!(
        assembled("<div><span></span> <span></span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\"),
    _createTextVNode(),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn v_once_wraps_the_native_vnode_in_the_render_cache() {
    assert_eq!(
        assembled("<div v-once>x</div>"),
        "\
const { createElementVNode: _createElementVNode, createTextVNode: _createTextVNode, setBlockTracking: _setBlockTracking } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _cache[0] || (
    _setBlockTracking(-1, true),
    (_cache[0] = _createElementVNode(\"div\", null, [
      _createTextVNode(\"x\")
    ])).cacheIndex = 0,
    _setBlockTracking(1),
    _cache[0]
  )
}"
    );
}
