//! A directive resolved by name is declared once at the top of `ssrRender`
//! (`const _directive_focus = _resolveDirective("focus")`), as Vue's SSR
//! output and the client output declare it. Nuxt's directive auto-import
//! rewrites only that declaration into an import.

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_l0::{Allocator, String};

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

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
fn a_named_directive_is_declared_once_beside_v_show_and_inside_v_for() {
    let code = render(
        r#"<input v-focus v-show="ok" /><div v-focus:a.b="ok">x</div><span v-for="i in items" :key="i" v-focus v-tip="i">{{ i }}</span>"#,
        &SsrCompilerOptions::default(),
    );
    assert_eq!(
        code,
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  const _directive_focus = _resolveDirective("focus")
  const _directive_tip = _resolveDirective("tip")
  _push(`<!--[--><input${_ssrRenderAttrs(_mergeProps({ style: ((_ctx.ok) ? null : { display: "none" }) }, _ssrGetDirectiveProps(_ctx, _directive_focus)))}><div${_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directive_focus, _ctx.ok, "a", { b: true }))}>x</div>`)
  _push(`<!--[-->`)
  _ssrRenderList(_ctx.items, (i) => {
    _push(`<span${_ssrRenderAttrs(_mergeProps({ key: i }, _ssrGetDirectiveProps(_ctx, _directive_focus), _ssrGetDirectiveProps(_ctx, _directive_tip, i)))}>${_ssrInterpolate(i)}</span>`)
  })
  _push(`<!--]-->`)
  _push(`<!--]-->`)
}
"#
    );
}

#[test]
fn directive_declarations_precede_temps_and_css_variables() {
    let code = render(
        r#"<div v-click-outside v-html-ish></div>"#,
        &SsrCompilerOptions {
            ssr_css_vars: Some("{ \"--c\": (_ctx.c) }".into()),
            ..Default::default()
        },
    );
    assert_eq!(
        code,
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  const _directive_click_outside = _resolveDirective("click-outside")
  const _directive_html_ish = _resolveDirective("html-ish")
  const _cssVars = { style: { "--c": (_ctx.c) } }
  _attrs = _mergeProps(_attrs, _cssVars)
  let _temp0
  _push(`<div${_ssrRenderAttrs(_temp0 = _mergeProps(_attrs, _ssrGetDirectiveProps(_ctx, _directive_click_outside), _ssrGetDirectiveProps(_ctx, _directive_html_ish)))}>${("textContent" in _temp0) ? _ssrInterpolate(_temp0.textContent) : _temp0.innerHTML ?? ''}</div>`)
}
"#
    );
}

#[test]
fn setup_and_imported_directives_keep_their_binding_reference() {
    let mut metadata = BindingMetadata {
        is_script_setup: true,
        ..Default::default()
    };
    metadata
        .bindings
        .insert("vFocus".into(), BindingType::SetupConst);
    metadata
        .bindings
        .insert("vTip".into(), BindingType::SetupMaybeRef);
    let source = r#"<p v-focus v-tip="1" v-other>x</p>"#;
    for (inline, expected) in [
        (
            false,
            r#"function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  const _directive_other = _resolveDirective("other")
  _push(`<p${_ssrRenderAttrs(_mergeProps(_attrs, _ssrGetDirectiveProps(_ctx, $setup["vFocus"]), _ssrGetDirectiveProps(_ctx, $setup["vTip"], 1), _ssrGetDirectiveProps(_ctx, _directive_other)))}>x</p>`)
}
"#,
        ),
        (
            true,
            r#"function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  const _directive_other = _resolveDirective("other")
  _push(`<p${_ssrRenderAttrs(_mergeProps(_attrs, _ssrGetDirectiveProps(_ctx, vFocus), _ssrGetDirectiveProps(_ctx, _unref(vTip), 1), _ssrGetDirectiveProps(_ctx, _directive_other)))}>x</p>`)
}
"#,
        ),
    ] {
        let options = SsrCompilerOptions {
            inline,
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        };
        assert_eq!(render(source, &options), expected, "inline={inline}");
    }
}

#[test]
fn a_template_without_named_directives_declares_nothing() {
    let code = render(
        r#"<div v-show="ok" :class="c">{{ msg }}</div>"#,
        &SsrCompilerOptions::default(),
    );
    assert_eq!(
        code,
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_mergeProps({ class: _ctx.c }, _attrs, { style: ((_ctx.ok) ? null : { display: "none" }) }))}>${_ssrInterpolate(_ctx.msg)}</div>`)
}
"#
    );
}
