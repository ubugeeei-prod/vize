use super::*;

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
fn v_once_single_space_between_nodes_keeps_the_string_argument() {
    assert_eq!(
        assembled(r#"<div v-once><b>{{ title }}</b> <br><span v-html="html"></span></div>"#),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, createTextVNode: _createTextVNode, setBlockTracking: _setBlockTracking } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"br\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _cache[0] || (
    _setBlockTracking(-1, true),
    (_cache[0] = _createElementVNode(\"div\", null, [
      _createElementVNode(\"b\", null, _toDisplayString(title), 1 /* TEXT */),
      _createTextVNode(\" \"),
      _hoisted_1,
      _createElementVNode(\"span\", { innerHTML: html }, null, 8 /* PROPS */, [\"innerHTML\"])
    ])).cacheIndex = 0,
    _setBlockTracking(1),
    _cache[0]
  )
}"
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
