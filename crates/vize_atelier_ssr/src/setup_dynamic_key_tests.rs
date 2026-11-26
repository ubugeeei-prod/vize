//! A dynamic attribute name is a template expression like any other, so a
//! `<script setup>` binding or a prop resolves through the binding table
//! (`$setup.attr`, `$props.pk`) instead of `_ctx`, which the instance proxy
//! never fills for setup bindings. Hydration does not patch attributes, so a
//! key dropped here never reaches the client either.

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_l0::{Allocator, String};

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

const SOURCE: &str = r#"<p :[attr]="'x'">a</p><p :[r]="'y'">b</p><p :[pk]="'z'">c</p><p :[attr].camel="'w'">d</p><Comp :[attr]="'x'" :[r].camel="'y'" @[evt]="go" /><i v-for="k in keys" :[k]="1" :[free]="2"></i>"#;

fn options(inline: bool) -> SsrCompilerOptions {
    let mut metadata = BindingMetadata {
        is_script_setup: true,
        ..Default::default()
    };
    for (name, kind) in [
        ("attr", BindingType::SetupConst),
        ("r", BindingType::SetupRef),
        ("pk", BindingType::Props),
        ("evt", BindingType::SetupConst),
        ("go", BindingType::SetupConst),
        ("keys", BindingType::SetupConst),
    ] {
        metadata.bindings.insert(name.into(), kind);
    }
    SsrCompilerOptions {
        inline,
        binding_metadata: Some(metadata),
        ..Default::default()
    }
}

/// The render function both lanes emit, which must agree.
fn render(source: &str, options: &SsrCompilerOptions) -> String {
    let compile = |lane| {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            SsrCompilerExperimentalOptions::default(),
            true,
            lane,
        );
        assert_eq!(errors.len(), 0, "{source}: {errors:?}");
        result.code
    };
    let selected = compile(SsrLane::Selected);
    assert_eq!(selected, compile(SsrLane::LegacyOnly), "{source}");
    selected
}

#[test]
fn setup_bindings_and_props_name_dynamic_attributes_through_the_binding_table() {
    assert_eq!(
        render(SOURCE, &options(false)),
        r#"function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(`<!--[--><p${_ssrRenderAttrs({ [$setup.attr || ""]: 'x' })}>a</p><p${_ssrRenderAttrs({ [$setup.r || ""]: 'y' })}>b</p><p${_ssrRenderAttrs({ [$props.pk || ""]: 'z' })}>c</p><p${_ssrRenderAttrs({ [_camelize(($setup.attr) || "") || ""]: 'w' })}>d</p>`)
  _push(_ssrRenderComponent(_resolveComponent("Comp"), _mergeProps(_normalizeProps({ [$setup.attr || ""]: 'x' }), _normalizeProps({ [_camelize(($setup.r) || "") || ""]: 'y' }), _normalizeProps({ [_toHandlerKey($setup.evt) || ""]: $setup.go })), null, _parent))
  _push(`<!--[-->`)
  _ssrRenderList($setup.keys, (k) => {
    _push(`<i${_ssrRenderAttrs({ [k || ""]: 1, [_ctx.free || ""]: 2 })}></i>`)
  })
  _push(`<!--]-->`)
  _push(`<!--]-->`)
}
"#
    );
}

#[test]
fn inline_setup_bindings_name_dynamic_attributes_directly() {
    assert_eq!(
        render(SOURCE, &options(true)),
        r#"function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(`<!--[--><p${_ssrRenderAttrs({ [attr || ""]: 'x' })}>a</p><p${_ssrRenderAttrs({ [r.value || ""]: 'y' })}>b</p><p${_ssrRenderAttrs({ [__props.pk || ""]: 'z' })}>c</p><p${_ssrRenderAttrs({ [_camelize((attr) || "") || ""]: 'w' })}>d</p>`)
  _push(_ssrRenderComponent(_resolveComponent("Comp"), _mergeProps(_normalizeProps({ [attr || ""]: 'x' }), _normalizeProps({ [_camelize((r.value) || "") || ""]: 'y' }), _normalizeProps({ [_toHandlerKey(evt) || ""]: go })), null, _parent))
  _push(`<!--[-->`)
  _ssrRenderList(keys, (k) => {
    _push(`<i${_ssrRenderAttrs({ [k || ""]: 1, [_ctx.free || ""]: 2 })}></i>`)
  })
  _push(`<!--]-->`)
  _push(`<!--]-->`)
}
"#
    );
}

#[test]
fn a_template_without_bindings_reads_dynamic_attributes_from_ctx() {
    assert_eq!(
        render(
            r#"<p :[attr]="'x'" :[name].camel="1">a</p>"#,
            &SsrCompilerOptions::default()
        ),
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<p${_ssrRenderAttrs(_mergeProps({ [_ctx.attr || ""]: 'x', [_camelize((_ctx.name) || "") || ""]: 1 }, _attrs))}>a</p>`)
}
"#
    );
}
