//! `v-once` on a native element caches the vnode it renders without `v-once`,
//! minus handler caching (#10875). Both lanes dropped its listeners, `v-html`,
//! runtime directives and every patch flag but CLASS/STYLE. Expected
//! initializers are `@vue/compiler-sfc` 3.5.44 `compileTemplate` bytes
//! (function mode, `with (_ctx)` indent removed).

use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_legacy_with_options,
};
use vize_l0::Allocator;

const CASES: &[(&str, &str)] = &[
    (
        r#"<button v-once :title="String(n)" @click="inc">{{ n }}</button>"#,
        r#"_createElementVNode("button", {
      title: String(n),
      onClick: inc
    }, [
      _createTextVNode(_toDisplayString(n), 1 /* TEXT */)
    ], 8 /* PROPS */, ["title", "onClick"])"#,
    ),
    (
        r#"<span v-once class="a" :class="{ on: ok }" @click.stop="go()">{{ n }} items</span>"#,
        r#"_createElementVNode("span", {
      class: _normalizeClass(["a", { on: ok }]),
      onClick: _withModifiers($event => (go()), ["stop"])
    }, [
      _createTextVNode(_toDisplayString(n) + " items", 1 /* TEXT */)
    ], 10 /* CLASS, PROPS */, ["onClick"])"#,
    ),
    (
        r#"<div v-once v-html="raw"></div>"#,
        r#"_createElementVNode("div", { innerHTML: raw }, null, 8 /* PROPS */, ["innerHTML"])"#,
    ),
    (
        r#"<input v-once v-model="m" @keyup.enter="go">"#,
        r#"_withDirectives(_createElementVNode("input", {
      "onUpdate:modelValue": $event => ((m) = $event),
      onKeyup: _withKeys(go, ["enter"])
    }, null, 40 /* PROPS, NEED_HYDRATION */, ["onUpdate:modelValue", "onKeyup"]), [
      [_vModelText, m]
    ])"#,
    ),
    (
        r#"<p v-once v-show="ok" id="p">{{ n }}</p>"#,
        r#"_withDirectives(_createElementVNode("p", { id: "p" }, [
      _createTextVNode(_toDisplayString(n), 1 /* TEXT */)
    ], 512 /* NEED_PATCH */), [
      [_vShow, ok]
    ])"#,
    ),
    (
        // Vue hoists the dynamic prop list as `_hoisted_1`; Vize keeps it inline.
        r#"<div v-once><button @click="go">{{ n }}</button></div>"#,
        r#"_createElementVNode("div", null, [
      _createElementVNode("button", { onClick: go }, _toDisplayString(n), 9 /* TEXT, PROPS */, ["onClick"])
    ])"#,
    ),
];

fn initializer(code: &str) -> Option<&str> {
    let (_, cached) = code.split_once("(_cache[0] = ")?;
    let (vnode, _) = cached.split_once(").cacheIndex = 0,")?;
    Some(vnode)
}

#[test]
fn element_v_once_keeps_listeners_props_and_directives_in_both_lanes() {
    for (source, expected) in CASES {
        let allocator = Allocator::new();
        let (_, errors, l2) = compile_template(&allocator, source);
        assert_eq!(errors.len(), 0, "{source}");
        let allocator = Allocator::new();
        let (_, errors, legacy) =
            compile_template_legacy_with_options(&allocator, source, DomCompilerOptions::default());
        assert_eq!(errors.len(), 0, "{source}");
        assert_eq!(initializer(&l2.code), Some(*expected), "L2: {source}");
        assert_eq!(
            initializer(&legacy.code),
            Some(*expected),
            "legacy: {source}"
        );
    }
}
