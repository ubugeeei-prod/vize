//! Vue SSR compiler for Vize.
//!
//! This module provides SSR-specific compilation including:
//! - SSR code generation with template literals and `_push()` calls
//! - SSR-specific directive transforms (v-model, v-show)
//! - SSR slot rendering
//! - SSR component rendering
//! - SSR teleport and suspense handling
//!
//! **Atelier** (/ˌætəlˈjeɪ/) is an artist's workshop or studio. The "ssr" atelier
//! specializes in server-side rendering output, producing HTML strings instead of
//! VNode trees.

#![allow(clippy::collapsible_match)]
#![cfg_attr(test, allow(clippy::disallowed_macros))]

pub mod codegen;
mod compile;
pub mod errors;
pub mod options;
mod stage_options;
pub mod steps;

pub use codegen::{SsrCodegenContext, SsrCodegenResult};
#[allow(deprecated)]
pub use compile::compile_ssr_with_vue_parser_quirks;
pub use compile::{
    compile_ssr, compile_ssr_with_custom_elements_and_template_syntax, compile_ssr_with_options,
    compile_ssr_with_template_syntax,
};
pub use errors::SsrErrorCode;
pub use options::SsrCompilerOptions;
pub use steps::{
    get_v_html_exp, get_v_model_exp, get_v_show_exp, get_v_text_exp, has_v_html, has_v_model,
    has_v_show, has_v_text,
};

// Re-export core types
pub use vize_atelier_core::{
    Allocator, CompilerError, Namespace, RootNode, RuntimeHelper, TemplateChildNode,
    codegen as core_codegen, errors as core_errors, lane, parser, runtime_helpers, tokenizer,
    transform,
};

#[cfg(test)]
mod tests {
    use super::{
        SsrCompilerOptions, compile_ssr, compile_ssr_with_options, compile_ssr_with_template_syntax,
    };
    use vize_atelier_core::TemplateSyntaxMode;
    use vize_s0::Allocator;

    #[test]
    fn test_compile_simple_element() {
        let allocator = Allocator::new();
        let (root, errors, result) = compile_ssr(&allocator, "<div>hello</div>");

        assert!(errors.is_empty());
        assert_eq!(root.children.len(), 1);
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_interpolation() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, "<div>{{ msg }}</div>");

        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_scoped_dynamic_component_keeps_scope_id() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_options(
            &allocator,
            r#"<component :is="tag"><span>Logo</span></component>"#,
            SsrCompilerOptions {
                scope_id: Some("data-v-test".into()),
                ..SsrCompilerOptions::default()
            },
        );

        assert!(errors.is_empty());
        assert!(
            result
                .code
                .contains(r#"_mergeProps({  }, { "data-v-test": "" })"#)
                || result.code.contains(r#"{ "data-v-test": "" }"#),
            "{}",
            result.code
        );
        assert!(
            result
                .code
                .contains(r#"_createElementVNode("span", { "data-v-test": "" }"#),
            "{}",
            result.code
        );
    }

    #[test]
    fn test_scoped_component_keeps_scope_id() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_options(
            &allocator,
            r#"<NuxtLink to="/news" class="news__link"><span>News</span></NuxtLink>"#,
            SsrCompilerOptions {
                scope_id: Some("data-v-news".into()),
                ..SsrCompilerOptions::default()
            },
        );

        assert!(errors.is_empty());
        assert!(
            result.code.contains(r#""data-v-news": """#),
            "{}",
            result.code
        );
        assert!(
            result.code.contains(r#"class: "news__link""#),
            "{}",
            result.code
        );
    }

    #[test]
    fn test_compile_template_syntax_quirks_accepts_invalid_html_self_closing() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_template_syntax(
            &allocator,
            "<div /><span></span>",
            Default::default(),
            TemplateSyntaxMode::Quirks,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert!(!result.code.is_empty());
    }

    #[test]
    fn test_compile_standard_warns_and_rewrites_invalid_html_self_closing() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, "<div /><span></span>");

        assert!(errors.iter().any(|error| error.is_recoverable()));
        assert!(!result.code.is_empty());
    }

    #[test]
    fn test_compile_strict_rejects_invalid_html_self_closing() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_template_syntax(
            &allocator,
            "<div /><span></span>",
            Default::default(),
            TemplateSyntaxMode::Strict,
        );

        assert!(errors.iter().any(|error| !error.is_recoverable()));
        assert!(result.code.is_empty());
    }

    #[test]
    fn test_ssr_v_model_textarea_renders_bound_value() {
        // Regression for #962: `<textarea v-model="x">` must render `x` as
        // escaped text content. The previous SSR path emitted
        // `<textarea></textarea>` with no body, losing the initial value
        // and triggering hydration mismatches.
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<textarea v-model="x"></textarea>"#);
        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            result.code.contains("_ssrInterpolate(_ctx.x)"),
            "expected textarea body to interpolate the model value, got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_v_model_select_marks_matching_option_selected() {
        // Regression for #962: `<select v-model="x">` must render the
        // matching `<option>` with `selected` set, not silently drop the
        // bound value.
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<select v-model="x"><option value="a">A</option><option value="b">B</option></select>"#,
        );
        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            result.code.contains("_ssrLooseEqual(_ctx.x, \"a\")"),
            "expected loose-equal for option a, got:\n{}",
            result.code
        );
        assert!(
            result.code.contains("_ssrLooseEqual(_ctx.x, \"b\")"),
            "expected loose-equal for option b, got:\n{}",
            result.code
        );
        assert!(
            result.code.contains("\" selected\""),
            "expected ` selected` literal, got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_dynamic_slot_outlet_name_stays_expression() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_options(
            &allocator,
            r#"<Parent><slot :name="((item.slot || 'item') as keyof Slots)" :item="item" /></Parent>"#,
            SsrCompilerOptions {
                is_ts: true,
                ..SsrCompilerOptions::default()
            },
        );

        assert!(errors.is_empty());
        assert!(
            result
                .code
                .contains(r#"_ssrRenderSlot(_ctx.$slots, _ctx.item.slot || "item""#),
            "{}",
            result.code
        );
        assert!(
            result
                .code
                .contains(r#"_renderSlot(_ctx.$slots, _ctx.item.slot || "item""#),
            "{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_v_if_v_else() {
        let allocator = Allocator::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<div v-if="ok">yes</div><p v-else>no</p>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_for_list() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<ul><li v-for="item in items" :key="item.id">{{ item.name }}</li></ul>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_keyed_template_v_for_keeps_iteration_fragment() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Comp>
  <template v-for="item in items" :key="item.id">
    <span v-if="item.visible">{{ item.label }}</span>
    <span v-else>hidden</span>
  </template>
</Comp>"#,
        );

        assert!(errors.is_empty(), "{errors:?}");
        let list_start = result
            .code
            .find("_ssrRenderList(_ctx.items")
            .expect("expected ssr render list");
        let list_body = &result.code[list_start..];
        assert!(
            list_body.contains("_push(`<!--[-->`)"),
            "keyed template v-for iteration must render a Fragment boundary:\n{}",
            result.code
        );
        assert!(
            list_body.contains("_push(`<!--]-->`)"),
            "keyed template v-for iteration must close its Fragment boundary:\n{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("_createBlock(_Fragment, null, _renderList(_ctx.items"),
            "vnode fallback must wrap v-for in a Fragment block:\n{}",
            result.code
        );
        assert!(
            result.code.contains("return [(_openBlock(true)"),
            "slot functions must return vnode arrays:\n{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("_createBlock(_Fragment, { key: item.id }"),
            "vnode fallback must keep the keyed template iteration Fragment:\n{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_keyed_template_v_for_single_element_unwraps_iteration_fragment() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Comp>
  <template v-for="(item, index) in items" :key="index">
    <div class="item">{{ item.label }}</div>
  </template>
</Comp>"#,
        );

        assert!(errors.is_empty(), "{errors:?}");
        let list_start = result
            .code
            .find("_ssrRenderList(_ctx.items")
            .expect("expected ssr render list");
        let list_body = &result.code[list_start..];
        assert!(
            !list_body.contains("_push(`<!--[-->`)"),
            "single element template v-for iterations must not render an extra Fragment boundary:\n{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("_createElementVNode(\"div\", { key: index"),
            "vnode fallback must forward the template key to the unwrapped child:\n{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_keyed_template_v_for_slot_fallback_keeps_iteration_fragment() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Outer>
  <slot>
  <template v-for="displayed in [value]" :key="displayed">
    <span v-if="displayed">{{ displayed }}</span>
    <span v-else>empty</span>
  </template>
  </slot>
</Outer>"#,
        );

        assert!(errors.is_empty(), "{errors:?}");
        let list_start = result
            .code
            .find("_ssrRenderList([_ctx.value]")
            .expect("expected ssr render list");
        let list_body = &result.code[list_start..];
        assert!(
            list_body.contains("_push(`<!--[-->`)"),
            "keyed template v-for fallback iteration must render a Fragment boundary:\n{}",
            result.code
        );
        assert!(
            list_body.contains("_push(`<!--]-->`)"),
            "keyed template v-for fallback iteration must close its Fragment boundary:\n{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("_createBlock(_Fragment, null, _renderList([_ctx.value]"),
            "slot fallback vnode branch must wrap v-for in a Fragment block:\n{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("_createBlock(_Fragment, { key: displayed }"),
            "slot fallback vnode branch must keep the keyed template iteration Fragment:\n{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_static_and_dynamic_attrs() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<a class="link" :href="url" target="_blank">{{ label }}</a>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_bind_object() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<div v-bind="attrs">content</div>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_html() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<div v-html="raw"></div>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_dynamic_class_and_style() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<div :class="{ active: isActive }" :style="{ color }">x</div>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_component_with_props_and_slot() {
        let allocator = Allocator::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<MyCard :title="t"><p>body</p></MyCard>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_fragment_multiple_roots() {
        let allocator = Allocator::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<header>a</header><main>{{ b }}</main>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_text_and_interpolation_mix() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<p>Hello {{ name }}, you have {{ count }} items</p>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_show() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<div v-show="visible">toggle</div>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    // Regression: `<template v-for #[name]>` (dynamically-named looped slots, as
    // used by `@nuxt/ui`'s DashboardSearchButton) must compile to
    // `createSlots(base, [renderList(...)])`. Previously these slots were
    // collapsed into the component's `default` slot, dropping the named-slot
    // routing and leaking the scoped slot param as `_ctx.slotData`, which made
    // the SSR renderer read `.type` off an undefined vnode and return a 500.
    #[test]
    fn test_ssr_dynamic_v_for_slot_uses_create_slots() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Child>
  <template v-for="(_, name) in slots" #[name]="slotData">
    <slot :name="name" v-bind="slotData" />
  </template>
  <template #trailing="{ ui }">
    <div>{{ ui }}</div>
  </template>
</Child>"#,
        );
        assert!(errors.is_empty());
        // Must route the dynamic slots through createSlots, not `default`.
        assert!(
            result.code.contains("_createSlots("),
            "expected createSlots for dynamic v-for slot:\n{}",
            result.code
        );
        // The looped entry exposes `{ name, fn }` with the local `name` alias
        // (no `_ctx.` prefix) and the in-scope `slotData` param.
        assert!(
            result.code.contains("name,") || result.code.contains("name: name"),
            "expected local `name` alias in looped slot entry:\n{}",
            result.code
        );
        assert!(
            !result.code.contains("_ctx.slotData"),
            "scoped slot param `slotData` must not leak as `_ctx.slotData`:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }

    // Regression: a slot outlet's children are its fallback content; the
    // vnode branch emitted `_renderSlot(slots, name, props)` without the
    // fallback argument, so e.g. nuxt-ui Button's `<slot>{{ label }}</slot>`
    // label vanished whenever Button rendered through a parent's vnode
    // branch.
    #[test]
    fn test_ssr_slot_outlet_fallback_survives_vnode_branch() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Outer>
  <button>
    <slot :ui="ui">
      <span v-if="label">{{ label }}</span>
    </slot>
  </button>
</Outer>"#,
        );
        assert!(errors.is_empty());
        assert!(
            result
                .code
                .contains("_renderSlot(_ctx.$slots, \"default\", { ui: _ctx.ui }, () => ["),
            "vnode branch must pass the slot fallback:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }

    // Regression: `<template v-if #name>` conditional slots must also flow
    // through createSlots rather than collapse into the default slot.
    #[test]
    fn test_ssr_conditional_slot_uses_create_slots() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Child>
  <template v-if="ok" #header>
    <span>head</span>
  </template>
</Child>"#,
        );
        assert!(errors.is_empty());
        assert!(
            result.code.contains("_createSlots("),
            "expected createSlots for conditional slot:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }

    // Regression: `v-slot` directly on a component (`<Comp v-slot="{ item }">`)
    // was dropped entirely, so the slot body compiled its params against the
    // instance (`_ctx.item`) in both the push and vnode branches (nuxt-ui
    // `<ULink v-slot="{ active, ...slotProps }">` inside NavigationMenu).
    #[test]
    fn test_ssr_component_level_v_slot_binds_props() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Comp v-slot="{ item }">
  <span>{{ item.label }}</span>
</Comp>"#,
        );
        assert!(errors.is_empty());
        assert!(
            result.code.contains("default: _withCtx(({ item }"),
            "component-level v-slot must bind its props pattern:\n{}",
            result.code
        );
        assert!(
            !result.code.contains("_ctx.item"),
            "scoped slot param `item` must not leak as `_ctx.item`:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_typescript_scoped_slot_props_are_accepted() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_with_options(
            &allocator,
            r#"<Popover v-slot="{ open, close }: { open: boolean, close?: () => void }">
  {{ open }}
</Popover>"#,
            SsrCompilerOptions {
                is_ts: true,
                ..Default::default()
            },
        );

        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            result.code.contains("default: _withCtx(({ open, close }"),
            "SSR slot props should be stripped to runtime params:\n{}",
            result.code
        );
    }

    #[test]
    fn test_ssr_forwarded_slot_flags_match_vue_shape() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<NuxtLink v-slot="{ href, navigate, route: linkRoute, isActive, isExactActive, ...rest }" v-bind="nuxtLinkProps" :to="to" custom>
  <template v-if="custom">
    <slot
      v-bind="{
        ...$attrs,
        as,
        type,
        disabled,
        href,
        navigate,
        active: isLinkActive({ route: linkRoute, isActive, isExactActive })
      }"
    />
  </template>
  <ULinkBase v-else v-bind="{ as, type, disabled, href, navigate }">
    <slot :active="isLinkActive({ route: linkRoute, isActive, isExactActive })" />
  </ULinkBase>
</NuxtLink>"#,
        );
        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            result.code.contains("_: 3 /* FORWARDED */"),
            "top-level slot forwarding should be marked FORWARDED:\n{}",
            result.code
        );
        assert!(
            result.code.contains("_: 2 /* DYNAMIC */"),
            "slot forwarding nested inside a slot scope should be marked DYNAMIC:\n{}",
            result.code
        );
        assert!(
            !result
                .code
                .contains("_createVNode(_Fragment, null, [_renderSlot"),
            "single-child template branches should not add a Fragment around renderSlot:\n{}",
            result.code
        );
        assert!(
            result.code.contains("_push, _parent, _scopeId)"),
            "slot outlets rendered inside slot functions should receive _scopeId:\n{}",
            result.code
        );
        assert!(
            result.code.contains("_parent, _scopeId))"),
            "components rendered inside slot functions should receive _scopeId:\n{}",
            result.code
        );
    }

    // Regression: static named slots with slot props must keep their own slot
    // entry (with the props pattern bound) in the vnode fallback branch of a
    // nested component. Collapsing them into `default: _withCtx(() => ...)`
    // compiles the body against the instance, so `collapsed` resolves to
    // undefined at runtime (nuxt-ui `<UDashboardSidebar>` inside
    // `<UDashboardGroup>`: `#header="{ collapsed }"`).
    #[test]
    fn test_ssr_named_scoped_slot_keeps_props_in_vnode_fallback() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Outer>
  <Inner>
    <template #header="{ collapsed }">
      <span>{{ collapsed }}</span>
    </template>
    <template #default="{ collapsed }">
      <Leaf :collapsed="collapsed" />
    </template>
  </Inner>
</Outer>"#,
        );
        assert!(errors.is_empty());
        assert!(
            result.code.contains("header: _withCtx(({ collapsed })"),
            "vnode fallback must bind the header slot props:\n{}",
            result.code
        );
        assert!(
            !result.code.contains("_ctx.collapsed"),
            "scoped slot param `collapsed` must not leak as `_ctx.collapsed`:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }

    // Regression: when a component with dynamic slots is nested inside another
    // component's slot, its vnode (client-render) fallback branch must also emit
    // `createSlots` rather than collapse the dynamic slots into `default`. This
    // mirrors `@nuxt/ui`'s `<DefineButtonTemplate><UButton><template v-for #[name]
    // />>` shape where both the push and fallback branches are generated.
    #[test]
    fn test_ssr_dynamic_slot_vnode_fallback_uses_create_slots() {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<Outer>
  <Inner>
    <template v-for="(_, name) in slots" #[name]="slotData">
      <slot :name="name" v-bind="slotData" />
    </template>
    <template #trailing>x</template>
  </Inner>
</Outer>"#,
        );
        assert!(errors.is_empty());
        // The nested Inner component is emitted both in the push branch and in
        // the vnode fallback (`else { return [...] }`) of Outer's default slot;
        // both must use createSlots, never `_ctx.slotData`.
        assert!(
            result.code.matches("_createSlots(").count() >= 2,
            "expected createSlots in both push and vnode fallback branches:\n{}",
            result.code
        );
        assert!(
            !result.code.contains("_ctx.slotData"),
            "scoped slot param must not leak as `_ctx.slotData`:\n{}",
            result.code
        );
        insta::assert_snapshot!(result.code.as_str());
    }
}
