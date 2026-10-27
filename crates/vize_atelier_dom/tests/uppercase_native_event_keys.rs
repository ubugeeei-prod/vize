//! Native `v-if` branch and `v-for` item roots keep an uppercase event name
//! as an `on:` key, like any other native element, and the key matches the
//! dynamic prop list (#10826). Keys and flags follow `@vue/compiler-sfc`
//! 3.5.44; a component keeps the camelized `onFooBar`.

use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_legacy_with_options,
};
use vize_l0::Allocator;

const CASES: &[(&str, &str)] = &[
    (
        r#"<div v-if="ok" @fooBar="pingA">x</div>"#,
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock("div", {
      key: 0,
      "on:fooBar": pingA
    }, "x", 40 /* PROPS, NEED_HYDRATION */, ["on:fooBar"]))
    : _createCommentVNode("v-if", true)
}"#,
    ),
    (
        r#"<div><div v-for="i in 3" :key="i" @fooBar="pingA">a</div></div>"#,
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, [
    (_openBlock(), _createElementBlock(_Fragment, null, _renderList(3, (i) => {
      return _createElementVNode("div", {
        key: i,
        "on:fooBar": pingA
      }, "a", 40 /* PROPS, NEED_HYDRATION */, ["on:fooBar"])
    }), 64 /* STABLE_FRAGMENT */))
  ]))
}"#,
    ),
    (
        r#"<Comp v-if="ok" @fooBar="pingA" />"#,
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Comp = _resolveComponent("Comp")
  
  return (ok)
    ? (_openBlock(), _createBlock(_component_Comp, {
      key: 0,
      onFooBar: pingA
    }, null, 8 /* PROPS */, ["onFooBar"]))
    : _createCommentVNode("v-if", true)
}"#,
    ),
];

#[test]
fn branch_and_item_roots_keep_uppercase_native_event_keys_in_both_lanes() {
    for (source, expected) in CASES {
        let allocator = Allocator::new();
        let (_, errors, l2) = compile_template(&allocator, source);
        assert_eq!(errors.len(), 0, "{source}");
        let allocator = Allocator::new();
        let (_, errors, legacy) =
            compile_template_legacy_with_options(&allocator, source, DomCompilerOptions::default());
        assert_eq!(errors.len(), 0, "{source}");
        assert_eq!(l2.code.trim_end(), *expected, "L2: {source}");
        assert_eq!(legacy.code.trim_end(), *expected, "legacy: {source}");
    }
}
