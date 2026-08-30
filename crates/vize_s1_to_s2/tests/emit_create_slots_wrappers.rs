//! `createSlots` wrapper edge cases.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

/// Vue's extra `newline()` after `genAssets` leaves indent on the blank line.
fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

#[test]
fn unwrapped_if_with_two_nested_slots_keeps_both_vnodes() {
    assert_eq!(
        assembled(
            r#"<Foo><template v-if="ok"><template #header>h</template><template #footer>f</template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (ok)
        ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
          _createTextVNode(\"h\"),
          _createTextVNode(\"f\")
        ], 64 /* STABLE_FRAGMENT */))
        : _createCommentVNode(\"v-if\", true)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn unwrapped_for_with_two_nested_slots_keeps_both_vnodes() {
    assert_eq!(
        assembled(
            r#"<Foo><template v-for="i in n"><template #header>h</template><template #footer>f</template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode, renderList: _renderList, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
        return (_openBlock(), _createElementBlock(_Fragment, null, [
          _createTextVNode(\"h\"),
          _createTextVNode(\"f\")
        ], 64 /* STABLE_FRAGMENT */))
      }), 256 /* UNKEYED_FRAGMENT */))
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}
