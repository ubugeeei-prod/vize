//! Regression tests for component prop bags whose only entry is a spread.
//!
//! A component whose props are a single `v-bind="obj"` (or `v-on="obj"`)
//! collapses to exactly one prop segment. That segment must still reach the
//! emitted prop bag: dropping it silently strips every prop from the component,
//! which is how `<Icon v-bind="iconProps" />` inside Nuxt UI's `Icon.vue`
//! started server-rendering with `name` undefined and crashed SSR.

use vize_atelier_ssr::compile_ssr;
use vize_s0::{Allocator, String};

fn compile(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, src);
    assert!(errors.is_empty(), "Compilation errors: {errors:?}");
    result.code
}

#[test]
fn lone_v_bind_spread_reaches_the_component_prop_bag() {
    assert_eq!(
        compile(r#"<Icon v-bind="iconProps" />"#),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _mergeProps(_normalizeProps(_guardReactiveProps(_ctx.iconProps)), _attrs), \
         null, _parent))\n}\n"
    );
}

#[test]
fn lone_v_on_object_spread_reaches_the_component_prop_bag() {
    assert_eq!(
        compile(r#"<Icon v-on="handlers" />"#),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _mergeProps(_normalizeProps(_guardReactiveProps(_toHandlers(_ctx.handlers))), _attrs), \
         null, _parent))\n}\n"
    );
}

/// The exact shape of Nuxt UI's `src/runtime/components/Icon.vue` template.
#[test]
fn nuxt_ui_icon_template_forwards_its_spread_in_both_branches() {
    assert_eq!(
        compile(
            r#"<Icon v-if="typeof name === 'string'" v-bind="iconProps" /><component :is="name" v-else />"#
        ),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         if (typeof _ctx.name === 'string') {\n    \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _mergeProps(_normalizeProps(_guardReactiveProps(_ctx.iconProps)), _attrs), \
         null, _parent))\n  \
         } else {\n    \
         _ssrRenderVNode(_push, _createVNode(_resolveDynamicComponent(_ctx.name), _attrs, null), _parent)\n  \
         }\n}\n"
    );
}

#[test]
fn lone_spread_survives_inside_a_v_for_item() {
    assert_eq!(
        compile(r#"<Icon v-for="item in items" v-bind="item" />"#),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         _push(`<!--[-->`)\n  \
         _ssrRenderList(_ctx.items, (item) => {\n    \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _normalizeProps(_guardReactiveProps(item)), null, _parent))\n  \
         })\n  \
         _push(`<!--]-->`)\n}\n"
    );
}

#[test]
fn spread_followed_by_static_props_keeps_source_order() {
    assert_eq!(
        compile(r#"<Icon v-bind="iconProps" name="x" />"#),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _mergeProps(_mergeProps(_normalizeProps(_guardReactiveProps(_ctx.iconProps)), \
         { name: \"x\" }), _attrs), null, _parent))\n}\n"
    );
}

#[test]
fn lone_entries_segment_stays_unwrapped() {
    assert_eq!(
        compile(r#"<Icon name="x" />"#),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  \
         _push(_ssrRenderComponent(_resolveComponent(\"Icon\"), \
         _mergeProps({ name: \"x\" }, _attrs), null, _parent))\n}\n"
    );
}
