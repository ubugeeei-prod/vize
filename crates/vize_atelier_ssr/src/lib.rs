//! Vue SSR compiler for Vize.
//!
//! This module provides SSR-specific compilation including:
//! - SSR code generation with template literals and `_push()` calls
//! - SSR-specific directive transforms (v-model, v-show)
//! - SSR slot rendering
//! - SSR component rendering
//! - SSR teleport and suspense handling
//!
//! ## Name Origin
//!
//! **Atelier** (/ˌætəlˈjeɪ/) is an artist's workshop or studio. The "ssr" atelier
//! specializes in server-side rendering output, producing HTML strings instead of
//! VNode trees.

#![allow(clippy::collapsible_match)]
#![cfg_attr(test, allow(clippy::disallowed_macros))]

pub mod codegen;
pub mod errors;
pub mod options;
pub mod transforms;

pub use codegen::{SsrCodegenContext, SsrCodegenResult};
pub use errors::SsrErrorCode;
pub use options::SsrCompilerOptions;
pub use transforms::{
    get_v_html_exp, get_v_model_exp, get_v_show_exp, get_v_text_exp, has_v_html, has_v_model,
    has_v_show, has_v_text,
};

// Re-export core types
pub use vize_atelier_core::{
    Allocator, CompilerError, Namespace, RootNode, RuntimeHelper, TemplateChildNode, ast,
    codegen as core_codegen, errors as core_errors, parser, runtime_helpers, tokenizer, transform,
};

use vize_atelier_core::{
    options::{ParserOptions, TemplateSyntaxMode, TransformOptions},
    parser::parse_with_options_and_template_syntax,
    transform::{transform as do_transform, transform_with_template_syntax_quirks},
};
use vize_carton::{Bump, String, profile};

/// Compile a Vue template for SSR with default options
pub fn compile_ssr<'a>(
    allocator: &'a Bump,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_with_options(allocator, source, SsrCompilerOptions::default())
}

/// Compile a Vue template for SSR with custom options
pub fn compile_ssr_with_options<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, TemplateSyntaxMode::Standard)
}

/// Compile a Vue template for SSR with Vue parser quirk compatibility.
#[deprecated(note = "use compile_ssr_with_template_syntax instead")]
pub fn compile_ssr_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, TemplateSyntaxMode::Quirks)
}

/// Compile a Vue template for SSR with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_ssr_with_template_syntax<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(allocator, source, options, template_syntax)
}

fn compile_ssr_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    let codegen_options = options.clone();

    // Create parser options
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        ..ParserOptions::default()
    };

    // Parse
    let (mut root, errors) = profile!(
        "atelier.ssr.template.parse",
        parse_with_options_and_template_syntax(allocator, source, parser_opts, template_syntax)
    );

    // Parser-level diagnostics that are recoverable (e.g. duplicate
    // attribute) must NOT gate SSR codegen for the same reason as the
    // DOM compiler — see #958.
    let fatal_count = errors.iter().filter(|e| !e.is_recoverable()).count();
    if fatal_count > 0 {
        let codegen_result = SsrCodegenResult {
            code: String::default(),
            preamble: String::default(),
        };
        return (root, errors.to_vec(), codegen_result);
    }

    // Transform with SSR-specific settings
    // SSR always uses prefix identifiers and disables hoisting/caching
    let transform_opts = TransformOptions {
        prefix_identifiers: true, // SSR always uses prefix
        hoist_static: false,      // No hoisting in SSR
        cache_handlers: false,    // No caching in SSR
        scope_id: codegen_options.scope_id.clone(),
        ssr: true,
        is_ts: codegen_options.is_ts,
        inline: codegen_options.inline,
        custom_renderer: codegen_options.custom_renderer,
        binding_metadata: codegen_options.binding_metadata.clone(),
        ..Default::default()
    };
    let analysis = options.croquis.map(|c| &*allocator.alloc(*c));
    let template_syntax_quirks = template_syntax.is_quirks();
    let transform_errors = profile!(
        "atelier.ssr.template.transform",
        if template_syntax_quirks {
            transform_with_template_syntax_quirks(allocator, &mut root, transform_opts, analysis)
        } else {
            do_transform(allocator, &mut root, transform_opts, analysis)
        }
    );

    // Surface transform diagnostics (e.g. invalid expressions) alongside
    // parse errors instead of dropping them — same channel as the DOM
    // compiler.
    let mut errors = errors.to_vec();
    errors.extend(transform_errors);

    // SSR codegen
    let codegen_ctx = SsrCodegenContext::new(allocator, &codegen_options);
    let codegen_result = profile!("atelier.ssr.template.codegen", codegen_ctx.generate(&root));

    (root, errors, codegen_result)
}

/// Get the namespace for an element based on its parent
fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }

    // Inherit namespace from parent
    if let Some(parent_tag) = parent {
        if vize_carton::is_svg_tag(parent_tag) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_carton::is_math_ml_tag(parent_tag)
            && tag != "annotation-xml"
            && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }

    Namespace::Html
}

#[cfg(test)]
mod tests {
    use super::{
        Bump, SsrCompilerOptions, compile_ssr, compile_ssr_with_options,
        compile_ssr_with_template_syntax,
    };
    use vize_atelier_core::TemplateSyntaxMode;

    #[test]
    fn test_compile_simple_element() {
        let allocator = Bump::new();
        let (root, errors, result) = compile_ssr(&allocator, "<div>hello</div>");

        assert!(errors.is_empty());
        assert_eq!(root.children.len(), 1);
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_compile_interpolation() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(&allocator, "<div>{{ msg }}</div>");

        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_scoped_dynamic_component_keeps_scope_id() {
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(&allocator, "<div /><span></span>");

        assert!(errors.iter().any(|error| error.is_recoverable()));
        assert!(!result.code.is_empty());
    }

    #[test]
    fn test_compile_strict_rejects_invalid_html_self_closing() {
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<div v-if="ok">yes</div><p v-else>no</p>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_for_list() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<ul><li v-for="item in items" :key="item.id">{{ item.name }}</li></ul>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_static_and_dynamic_attrs() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<a class="link" :href="url" target="_blank">{{ label }}</a>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_bind_object() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<div v-bind="attrs">content</div>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_html() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(&allocator, r#"<div v-html="raw"></div>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_dynamic_class_and_style() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<div :class="{ active: isActive }" :style="{ color }">x</div>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_component_with_props_and_slot() {
        let allocator = Bump::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<MyCard :title="t"><p>body</p></MyCard>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_fragment_multiple_roots() {
        let allocator = Bump::new();
        let (_, errors, result) =
            compile_ssr(&allocator, r#"<header>a</header><main>{{ b }}</main>"#);
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_text_and_interpolation_mix() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_ssr(
            &allocator,
            r#"<p>Hello {{ name }}, you have {{ count }} items</p>"#,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_ssr_v_show() {
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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

    // Regression: static named slots with slot props must keep their own slot
    // entry (with the props pattern bound) in the vnode fallback branch of a
    // nested component. Collapsing them into `default: _withCtx(() => ...)`
    // compiles the body against the instance, so `collapsed` resolves to
    // undefined at runtime (nuxt-ui `<UDashboardSidebar>` inside
    // `<UDashboardGroup>`: `#header="{ collapsed }"`).
    #[test]
    fn test_ssr_named_scoped_slot_keeps_props_in_vnode_fallback() {
        let allocator = Bump::new();
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
        let allocator = Bump::new();
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
