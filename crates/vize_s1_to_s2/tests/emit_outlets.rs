//! Slot-outlet emit pins (`renderSlot` / `_: 3 FORWARDED`).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn assert_shipped_parity(source: &str) {
    assert_eq!(assembled(source), shipped(source), "{source}");
}

#[test]
fn a_bare_slot_uses_render_slot() {
    assert_eq!(
        assembled("<slot></slot>"),
        "\
const { renderSlot: _renderSlot } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\")
}"
    );
}

#[test]
fn a_named_slot_quotes_the_name() {
    assert_eq!(
        assembled(r#"<slot name="header"></slot>"#),
        "\
const { renderSlot: _renderSlot } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"header\")
}"
    );
}

#[test]
fn fallback_text_passes_an_empty_props_object() {
    assert_eq!(
        assembled("<slot>fallback</slot>"),
        "\
const { renderSlot: _renderSlot, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", {}, () => [
    _createTextVNode(\"fallback\")
  ])
}"
    );
}

#[test]
fn a_forwarded_outlet_sets_the_forwarded_flag() {
    assert_eq!(
        assembled("<Foo><slot></slot></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, renderSlot: _renderSlot, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _renderSlot(_ctx.$slots, \"default\")
    ]),
    _: 3 /* FORWARDED */
  }))
}")
    );
}

#[test]
fn dynamic_slot_name_with_forwarded_outlet_keeps_forwarded_flag() {
    assert_shipped_parity(
        r#"<Foo><template #[target]><Bar><slot name="selectors" /></Bar></template></Foo>"#,
    );
}

#[test]
fn fallback_interp_expands_the_compound_into_sibling_children() {
    assert_eq!(
        assembled("<slot>hello {{ msg }}</slot>"),
        "\
const { toDisplayString: _toDisplayString, renderSlot: _renderSlot, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", {}, () => [
    _createTextVNode(\"hello \"),
    _toDisplayString(msg)
  ])
}"
    );
}

#[test]
fn fallback_dynamic_element_caches_static_sibling_children() {
    assert_shipped_parity(
        r#"<div v-if="pending"><Loading /></div><div v-else-if="resolved"><slot :result="result as T"></slot></div><div v-else><div :class="$style.error"><slot name="error" :error="error"><div><i class="icon"></i> {{ message }}</div><Button @click="retry"><i class="retry"></i> {{ retryText }}</Button></slot></div></div>"#,
    );
}

#[test]
fn named_event_props_use_component_listener_casing() {
    assert_eq!(
        assembled(r#"<slot @pick="choose"></slot>"#),
        "\
const { renderSlot: _renderSlot } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", { onPick: choose })
}"
    );
}

#[test]
fn dynamic_event_props_use_to_handler_key() {
    assert_eq!(
        assembled(r#"<slot @[event].enter.stop="handler"></slot>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, renderSlot: _renderSlot, toHandlerKey: _toHandlerKey } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", { [_toHandlerKey(_ctx.event)]: _withKeys(_withModifiers(handler, [\"stop\"]), [\"enter\"]) })
}"
    );
}

#[test]
fn duplicate_event_props_keep_the_shipped_duplicate_key_shape() {
    assert_eq!(
        assembled(r#"<slot @click="a" @click.stop="b"></slot>"#),
        "\
const { withModifiers: _withModifiers, renderSlot: _renderSlot } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", { onClick: a, onClick: _withModifiers(b, [\"stop\"]) })
}"
    );
}

#[test]
fn a_v_if_outlet_keeps_the_unused_open_block_helper() {
    assert_eq!(
        assembled(r#"<slot v-if="ok"></slot>"#),
        "\
const { renderSlot: _renderSlot, openBlock: _openBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? _renderSlot(_ctx.$slots, \"default\", { key: 0 })
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn dynamic_class_slot_prop_normalizes_like_the_shipped_lane() {
    assert_shipped_parity(r#"<slot name="sidebar" :class="ppNs.e('sidebar')" />"#);
}

#[test]
fn branch_keys_in_named_slots_follow_the_slot_output_order() {
    assert_shipped_parity(
        r#"<Foo><Bar v-if="icon"></Bar><template #actions><Baz v-if="mode === 'switch'"></Baz><Qux v-else-if="mode === 'button'"></Qux><div v-else></div></template></Foo>"#,
    );
}
