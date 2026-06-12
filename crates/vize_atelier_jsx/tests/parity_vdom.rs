//! VDOM-backend JSX/TSX parity suite (Part of #1491).
//!
//! These cases mirror the reference areas of `@vue/babel-plugin-jsx` (elements,
//! attributes, children, control flow, slots, directives, events) but assert
//! **Vize's** VDOM codegen output structure — helper calls, patch flags, prop
//! shapes — rather than byte-for-byte babel parity, since Vize emits through its
//! own `vize_atelier_dom` codegen path.
//!
//! Backend separation: every failure here points at the **VDOM** lowering +
//! codegen path. The Vapor mirror lives in `parity_vapor.rs`; TSX-specific
//! parity lives in `parity_tsx.rs`. See `PARITY_INVENTORY.md` for the full
//! covered-vs-deferred matrix.

use vize_atelier_jsx::{DomCompileOptions, JsxLang, compile_to_dom};
use vize_carton::Bump;

/// Compile JSX to VDOM render code, asserting a single error-free component.
fn dom(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

// ---------------------------------------------------------------------------
// Category: elements / intrinsic vs component resolution / fragments
// ---------------------------------------------------------------------------

#[test]
fn intrinsic_element_uses_create_element_block() {
    let code = dom("const A = () => <div/>;");
    assert!(code.contains("_createElementBlock(\"div\")"), "{code}");
    assert!(code.contains("_openBlock()"), "{code}");
}

#[test]
fn component_is_resolved_and_created_as_block() {
    // PascalCase tag => resolveComponent + createBlock (not createElementBlock).
    let code = dom("const A = () => <Comp/>;");
    assert!(code.contains("_resolveComponent(\"Comp\")"), "{code}");
    assert!(code.contains("_createBlock(_component_Comp"), "{code}");
    assert!(!code.contains("_createElementBlock(\"Comp\""), "{code}");
}

#[test]
fn fragment_uses_stable_fragment_flag() {
    let code = dom("const A = () => <><a/><b/></>;");
    assert!(code.contains("_Fragment"), "{code}");
    assert!(code.contains("64 /* STABLE_FRAGMENT */"), "{code}");
}

#[test]
fn fragment_with_dynamic_child_keeps_stable_fragment() {
    // A root fragment with mixed static + interpolation children.
    let code = dom("const A = () => <><h1>a</h1><p>{x}</p></>;");
    assert!(code.contains("_Fragment"), "{code}");
    assert!(code.contains("64 /* STABLE_FRAGMENT */"), "{code}");
    assert!(code.contains("_toDisplayString(x)"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: attributes (static / dynamic / spread / boolean / namespaced /
// class / style) and their patch flags
// ---------------------------------------------------------------------------

#[test]
fn static_attributes_are_inlined_with_no_patch_flag() {
    // Purely static props => no patch flag, no dynamic-prop array.
    let code = dom("const A = () => <div class=\"a\" id=\"b\"/>;");
    assert!(code.contains("class: \"a\""), "{code}");
    assert!(code.contains("id: \"b\""), "{code}");
    assert!(!code.contains("/* PROPS */"), "{code}");
}

#[test]
fn boolean_attribute_lowers_to_empty_string_value() {
    let code = dom("const A = () => <input disabled/>;");
    assert!(code.contains("disabled: \"\""), "{code}");
}

#[test]
fn single_dynamic_bind_emits_props_flag_and_dynamic_key() {
    let code = dom("const A = () => <div id={x}/>;");
    assert!(code.contains("{ id: x }"), "{code}");
    assert!(code.contains("8 /* PROPS */"), "{code}");
    assert!(code.contains("[\"id\"]"), "{code}");
}

#[test]
fn multiple_dynamic_binds_collect_all_dynamic_keys() {
    let code = dom("const A = () => <div id={a} title={b}/>;");
    assert!(code.contains("8 /* PROPS */"), "{code}");
    assert!(code.contains("[\"id\", \"title\"]"), "{code}");
}

#[test]
fn spread_alone_uses_normalize_and_guard_with_full_props() {
    let code = dom("const A = () => <div {...attrs}/>;");
    assert!(
        code.contains("_normalizeProps(_guardReactiveProps(attrs))"),
        "{code}"
    );
    assert!(code.contains("16 /* FULL_PROPS */"), "{code}");
}

#[test]
fn spread_mixed_with_static_uses_merge_props() {
    let code = dom("const A = () => <div class=\"a\" {...attrs}/>;");
    assert!(
        code.contains("_mergeProps({ class: \"a\" }, attrs)"),
        "{code}"
    );
    assert!(code.contains("16 /* FULL_PROPS */"), "{code}");
}

#[test]
fn dynamic_class_is_normalized_with_class_flag() {
    let code = dom("const A = () => <div class={c}/>;");
    assert!(code.contains("_normalizeClass(c)"), "{code}");
    assert!(code.contains("2 /* CLASS */"), "{code}");
}

#[test]
fn array_class_binding_is_normalized() {
    let code = dom("const A = () => <div class={['a', b]}/>;");
    assert!(code.contains("_normalizeClass(['a', b])"), "{code}");
    assert!(code.contains("2 /* CLASS */"), "{code}");
}

#[test]
fn dynamic_style_is_normalized_with_style_flag() {
    let code = dom("const A = () => <div style={s}/>;");
    assert!(code.contains("_normalizeStyle(s)"), "{code}");
    assert!(code.contains("4 /* STYLE */"), "{code}");
}

#[test]
fn namespaced_colon_attribute_name_is_preserved() {
    let code = dom("const A = () => <use xlink:href=\"#id\"/>;");
    assert!(code.contains("\"xlink:href\": \"#id\""), "{code}");
}

#[test]
fn key_prop_does_not_become_a_dynamic_patch_prop() {
    // `key` is a reserved VNode prop, not a regular patch prop.
    let code = dom("const A = () => <div key={k}/>;");
    assert!(code.contains("key: k"), "{code}");
    assert!(!code.contains("/* PROPS */"), "{code}");
}

#[test]
fn ref_prop_emits_need_patch_flag() {
    let code = dom("const A = () => <div ref={r}/>;");
    assert!(code.contains("ref: r"), "{code}");
    assert!(code.contains("512 /* NEED_PATCH */"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: children (text / interpolation / mixed) and the TEXT patch flag
// ---------------------------------------------------------------------------

#[test]
fn static_text_child_has_no_text_flag() {
    let code = dom("const A = () => <div>hello</div>;");
    assert!(code.contains("\"hello\""), "{code}");
    assert!(!code.contains("/* TEXT */"), "{code}");
}

#[test]
fn interpolation_child_uses_to_display_string_with_text_flag() {
    let code = dom("const A = () => <div>{count}</div>;");
    assert!(code.contains("_toDisplayString(count)"), "{code}");
    assert!(code.contains("1 /* TEXT */"), "{code}");
}

#[test]
fn mixed_text_and_interpolation_concatenates_with_text_flag() {
    let code = dom("const A = () => <div>Hi {name}!</div>;");
    assert!(
        code.contains("\"Hi \" + _toDisplayString(name) + \"!\""),
        "{code}"
    );
    assert!(code.contains("1 /* TEXT */"), "{code}");
}

#[test]
fn jsx_free_identifiers_are_not_ctx_prefixed() {
    // JSX render fns close over setup scope; interpolations stay bare.
    let code = dom("const A = () => <div>{count}</div>;");
    assert!(!code.contains("_ctx.count"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: control flow — conditional (&&, ternary), list (.map)
// ---------------------------------------------------------------------------

#[test]
fn logical_and_jsx_child_becomes_v_if() {
    let code = dom("const A = () => <ul>{ok && <li/>}</ul>;");
    assert!(!code.contains("_toDisplayString"), "{code}");
    assert!(code.contains("_createElementBlock(\"li\""), "{code}");
    assert!(code.contains("_createCommentVNode(\"v-if\""), "{code}");
}

#[test]
fn ternary_jsx_arms_become_two_branch_v_if() {
    let code = dom("const A = () => <div>{ok ? <a/> : <b/>}</div>;");
    assert!(code.contains("_createElementBlock(\"a\""), "{code}");
    assert!(code.contains("_createElementBlock(\"b\""), "{code}");
    assert!(code.contains("key: 0"), "{code}");
    assert!(code.contains("key: 1"), "{code}");
}

#[test]
fn map_callback_becomes_v_for_with_unkeyed_fragment() {
    let code = dom("const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;");
    assert!(code.contains("_renderList(items"), "{code}");
    assert!(code.contains("256 /* UNKEYED_FRAGMENT */"), "{code}");
    assert!(code.contains("_toDisplayString(i)"), "{code}");
}

#[test]
fn directive_v_if_on_element_compiles_to_conditional() {
    let code = dom("const A = () => <div v-if={ok}>x</div>;");
    assert!(code.contains("(ok)"), "{code}");
    assert!(code.contains("_createCommentVNode"), "{code}");
}

#[test]
fn non_jsx_logical_and_stays_an_interpolation() {
    // `{a && b}` with no JSX is value coalescing, not conditional rendering.
    let code = dom("const A = () => <div>{a && b}</div>;");
    assert!(code.contains("_toDisplayString(a && b)"), "{code}");
    assert!(!code.contains("_createCommentVNode(\"v-if\""), "{code}");
}

// ---------------------------------------------------------------------------
// Category: directives — v-model (element/component/typed), v-show, v-html,
// v-text, custom directives
// ---------------------------------------------------------------------------

#[test]
fn v_model_on_input_expands_to_update_handler_and_directive() {
    let code = dom("const A = () => <input v-model={val}/>;");
    assert!(code.contains("\"onUpdate:modelValue\""), "{code}");
    assert!(code.contains("_vModelText"), "{code}");
    assert!(code.contains("_withDirectives"), "{code}");
}

#[test]
fn v_model_on_checkbox_uses_checkbox_runtime_directive() {
    let code = dom("const A = () => <input type=\"checkbox\" v-model={checked}/>;");
    assert!(code.contains("_vModelCheckbox"), "{code}");
    assert!(code.contains("\"onUpdate:modelValue\""), "{code}");
}

#[test]
fn v_model_on_component_uses_model_value_prop() {
    // On a component, v-model becomes `modelValue` + `onUpdate:modelValue`.
    let code = dom("const A = () => <Input v-model={val}/>;");
    assert!(code.contains("modelValue: val"), "{code}");
    assert!(code.contains("\"onUpdate:modelValue\""), "{code}");
    assert!(!code.contains("_vModelText"), "{code}");
}

#[test]
fn v_model_with_named_argument_targets_that_prop() {
    // `v-model:foo` => `foo` prop + `onUpdate:foo` handler.
    let code = dom("const A = () => <Comp v-model:foo={val}/>;");
    assert!(code.contains("foo: val"), "{code}");
    assert!(code.contains("\"onUpdate:foo\""), "{code}");
}

#[test]
fn v_show_keeps_element_and_applies_runtime_directive() {
    let code = dom("const A = () => <div v-show={ok}>x</div>;");
    assert!(code.contains("_vShow"), "{code}");
    assert!(code.contains("_withDirectives"), "{code}");
}

#[test]
fn v_html_lowers_to_inner_html_prop() {
    let code = dom("const A = () => <div v-html={raw}/>;");
    assert!(code.contains("innerHTML: raw"), "{code}");
    assert!(code.contains("8 /* PROPS */"), "{code}");
}

#[test]
fn v_text_lowers_to_text_content_prop() {
    let code = dom("const A = () => <div v-text={msg}/>;");
    assert!(
        code.contains("textContent: _toDisplayString(msg)"),
        "{code}"
    );
}

#[test]
fn custom_directive_resolves_and_applies() {
    let code = dom("const A = () => <div v-foo={bar}/>;");
    assert!(code.contains("_resolveDirective(\"foo\")"), "{code}");
    assert!(code.contains("_withDirectives"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: event handlers + option modifiers
// ---------------------------------------------------------------------------

#[test]
fn plain_event_handler_stays_a_bind_prop() {
    let code = dom("const A = () => <button onClick={h}/>;");
    assert!(code.contains("onClick: h"), "{code}");
}

#[test]
fn capture_option_modifier_yields_capture_listener_key_with_hydration_flag() {
    // `onClickCapture` is lowered to a `v-on` with a capture modifier; codegen
    // emits the suffixed key and a NEED_HYDRATION patch flag.
    let code = dom("const A = () => <button onClickCapture={h}/>;");
    assert!(code.contains("onClickCapture: h"), "{code}");
    assert!(code.contains("40 /* PROPS, NEED_HYDRATION */"), "{code}");
}

#[test]
fn once_option_modifier_yields_once_listener_key() {
    let code = dom("const A = () => <button onClickOnce={h}/>;");
    assert!(code.contains("onClickOnce: h"), "{code}");
}

#[test]
fn composed_passive_capture_yields_combined_listener_key() {
    let code = dom("const A = () => <input onInputPassiveCapture={h}/>;");
    assert!(code.contains("onInputPassiveCapture: h"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: object slots / scoped slots / default render-prop slot
// ---------------------------------------------------------------------------

#[test]
fn object_child_lowers_to_named_with_ctx_slots() {
    let code = dom("const A = () => <Comp>{{ header: () => <h1>Hi</h1> }}</Comp>;");
    assert!(code.contains("header: _withCtx"), "{code}");
    assert!(code.contains("_createElementVNode(\"h1\""), "{code}");
    assert!(code.contains("_: 1 /* STABLE */"), "{code}");
}

#[test]
fn render_prop_child_lowers_to_default_scoped_slot() {
    let code = dom("const A = () => <List>{(item) => <li>{item}</li>}</List>;");
    assert!(code.contains("default: _withCtx((item) =>"), "{code}");
    assert!(code.contains("_toDisplayString(item)"), "{code}");
    assert!(!code.contains("_ctx.item"), "{code}");
}

#[test]
fn scoped_named_slot_keeps_destructured_param_bare() {
    let code = dom("const A = () => <List>{{ item: ({ x }) => <li>{x}</li> }}</List>;");
    assert!(code.contains("item: _withCtx(({ x }) =>"), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

#[test]
fn plain_element_children_form_implicit_default_slot() {
    let code = dom("const A = () => <Card><h1>Title</h1></Card>;");
    assert!(code.contains("default: _withCtx(() =>"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: v-model modifier forms (babel-plugin-jsx parity — #1489/#1491).
//
// JSX attribute names cannot contain `.`, so @vue/babel-plugin-jsx expresses
// v-model modifiers two ways: an array value `{[val, ['trim']]}` and an
// underscore-suffixed name `v-model_lazy`. Both lower to a `model` directive
// with `modelModifiers` + a single clean `onUpdate:modelValue` handler.
// ---------------------------------------------------------------------------

#[test]
fn v_model_modifier_array_attaches_model_modifiers() {
    let code = dom("const A = () => <input v-model={[val, ['trim']]}/>;");
    // No malformed nested handler, no leftover array as the bound expression.
    assert!(!code.contains("$event => ($event =>"), "{code}");
    assert!(!code.contains("[val, ['trim']]"), "{code}");
    // Single clean update handler bound to the model expression.
    assert!(
        code.contains("\"onUpdate:modelValue\": $event => ((val) = $event)"),
        "{code}"
    );
    // `.trim` lands as a model modifier on the v-model directive entry.
    assert!(code.contains("{ trim: true }"), "{code}");
}

#[test]
fn v_model_modifier_array_single_element_has_no_modifiers() {
    // `{[val]}` is just the bound expression with no modifiers/arg.
    let code = dom("const A = () => <input v-model={[val]}/>;");
    assert!(!code.contains("$event => ($event =>"), "{code}");
    assert!(
        code.contains("\"onUpdate:modelValue\": $event => ((val) = $event)"),
        "{code}"
    );
    assert!(code.contains("[_vModelText, val]"), "{code}");
}

#[test]
fn v_model_modifier_array_with_component_arg_and_modifiers() {
    // `{[val, 'foo', ['trim']]}` — arg `foo` + `.trim` on a component v-model.
    let code = dom("const A = () => <Comp v-model={[val, 'foo', ['trim']]}/>;");
    assert!(!code.contains("$event => ($event =>"), "{code}");
    assert!(code.contains("foo: val"), "{code}");
    assert!(
        code.contains("\"onUpdate:foo\": $event => ((val) = $event)"),
        "{code}"
    );
    assert!(code.contains("fooModifiers: { trim: true }"), "{code}");
}

#[test]
fn v_model_underscore_suffix_attaches_lazy_modifier() {
    let code = dom("const A = () => <input v-model_lazy={val}/>;");
    // No bogus custom directive resolution for the `_lazy` suffix.
    assert!(
        !code.contains("_resolveDirective(\"model_lazy\")"),
        "{code}"
    );
    assert!(!code.contains("_directive_model_lazy"), "{code}");
    assert!(
        code.contains("\"onUpdate:modelValue\": $event => ((val) = $event)"),
        "{code}"
    );
    assert!(code.contains("{ lazy: true }"), "{code}");
}

#[test]
fn v_model_underscore_suffix_chains_multiple_modifiers() {
    let code = dom("const A = () => <input v-model_number_lazy={val}/>;");
    assert!(
        !code.contains("_resolveDirective(\"model_number_lazy\")"),
        "{code}"
    );
    assert!(code.contains("number: true"), "{code}");
    assert!(code.contains("lazy: true"), "{code}");
}
