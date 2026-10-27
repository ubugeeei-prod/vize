//! `v-once` on a component caches the vnode it renders without `v-once`,
//! minus handler caching (#10815). Both lanes dropped every prop, listener
//! and slot. Expected initializers are `@vue/compiler-sfc` 3.5.44
//! `compileTemplate` bytes (function mode, `with (_ctx)` indent removed).

use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_legacy_with_options,
    compile_template_with_options,
};
use vize_l0::Allocator;

const CASES: &[(&str, &str)] = &[
    (
        r#"<MyButton v-once :label="label" @ping="onPing">text {{ label }}</MyButton>"#,
        r#"_createVNode(_component_MyButton, {
      label: label,
      onPing: onPing
    }, {
      default: _withCtx(() => [
        _createTextVNode("text " + _toDisplayString(label), 1 /* TEXT */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "onPing"])"#,
    ),
    (
        r#"<MyButton v-once label="static" />"#,
        r#"_createVNode(_component_MyButton, { label: "static" })"#,
    ),
    (
        r#"<Foo v-once v-model="m" @click="go()" />"#,
        r#"_createVNode(_component_Foo, {
      modelValue: m,
      "onUpdate:modelValue": $event => ((m) = $event),
      onClick: $event => (go())
    }, null, 8 /* PROPS */, ["modelValue", "onUpdate:modelValue", "onClick"])"#,
    ),
    (
        r#"<Foo v-once v-bind="obj"><template #foo="{ a }">{{ a }}</template></Foo>"#,
        r#"_createVNode(_component_Foo, _normalizeProps(_guardReactiveProps(obj)), {
      foo: _withCtx(({ a }) => [
        _createTextVNode(_toDisplayString(a), 1 /* TEXT */)
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */)"#,
    ),
    (
        r#"<Foo v-once v-example="h" />"#,
        r#"_withDirectives(_createVNode(_component_Foo, null, null, 512 /* NEED_PATCH */), [
      [_directive_example, h]
    ])"#,
    ),
    (
        // Vue hoists the dynamic prop list as `_hoisted_1`; Vize keeps it inline.
        r#"<Foo v-once><button @click="go">b</button></Foo>"#,
        r#"_createVNode(_component_Foo, null, {
      default: _withCtx(() => [
        _createElementVNode("button", { onClick: go }, "b", 8 /* PROPS */, ["onClick"])
      ]),
      _: 1 /* STABLE */
    })"#,
    ),
];

fn initializer(code: &str) -> Option<&str> {
    let (_, cached) = code.split_once("(_cache[0] = ")?;
    let (vnode, _) = cached.split_once(").cacheIndex = 0,")?;
    Some(vnode)
}

#[test]
fn component_v_once_keeps_props_listeners_and_slots_in_both_lanes() {
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

#[test]
fn cached_component_props_do_not_shift_static_slot_hoists() {
    // Minimized from the pinned Vue Material icon corpus: the component's
    // class stays in the cache while its SVG slot can still be hoisted.
    let source = r#"<md-icon class="md-icon-image" v-once><svg><path d="x" /></svg></md-icon>"#;
    for prefix_identifiers in [false, true] {
        for hoist_static in [false, true] {
            let options = DomCompilerOptions {
                prefix_identifiers,
                hoist_static,
                ..DomCompilerOptions::default()
            };
            let allocator = Allocator::new();
            let (_, errors, selected) =
                compile_template_with_options(&allocator, source, options.clone());
            assert_eq!(errors.len(), 0);
            let allocator = Allocator::new();
            let (_, errors, legacy) =
                compile_template_legacy_with_options(&allocator, source, options);
            assert_eq!(errors.len(), 0);
            assert_eq!(selected.code, legacy.code);
        }
    }
}
