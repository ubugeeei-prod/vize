//! `v-models` lowering (#3418).
//!
//! `v-models` is a `@vue/babel-plugin-jsx` built-in, not a user directive.
//! Before this was implemented it fell through the generic custom-directive path
//! and compiled to `resolveDirective("models")` — a lookup Vue resolves to
//! nothing at runtime, so the component rendered with **no model bindings at
//! all**, with no error and no warning.
//!
//! The expected output is pinned against the real plugin by the differential
//! oracle (`babel_compat/fixtures/babel-output.json`, rows `directives/v_models`,
//! `directives/v_models_mods`, `errors/v_models_not_array` and
//! `errors/v_models_entry_not_array`); this file asserts the full Vize output for
//! each shape plus the diagnostics for everything Vize refuses to lower.

mod common;

use vize_atelier_jsx::{JsxLang, VdomCompileOptions, compile_to_vdom, lower_source};
use vize_s0::Allocator;

use common::{find_directive, lower_one, root_element, simple_content};

fn errors(source: &str) -> Vec<String> {
    let bump = Allocator::new();
    let out = lower_source(&bump, bump.as_oxc(), source, JsxLang::Jsx);
    out.diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .map(|diagnostic| diagnostic.message.as_str().to_string())
        .collect()
}

fn render_code(source: &str) -> String {
    let bump = Allocator::new();
    let out = compile_to_vdom(&bump, source, JsxLang::Jsx, VdomCompileOptions::default());
    out.components
        .into_iter()
        .map(|component| component.code.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn single_entry_binds_model_value() {
    assert_eq!(
        render_code("const A = () => <B v-models={[[foo]]}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, {\n    \
         modelValue: foo,\n    \
         \"onUpdate:modelValue\": $event => ((foo) = $event)\n  \
         }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))\n}"
    );
}

#[test]
fn several_entries_each_become_one_binding() {
    // The corpus row `directives/v_models`: babel emits
    // `{"modelValue": foo, "onUpdate:modelValue": …, 'bar': bar, "onUpdate:bar": …}`.
    assert_eq!(
        render_code("const A = () => <B v-models={[[foo], [bar, 'bar']]}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, {\n    \
         modelValue: foo,\n    \
         \"onUpdate:modelValue\": $event => ((foo) = $event),\n    \
         bar: bar,\n    \
         \"onUpdate:bar\": $event => ((bar) = $event)\n  \
         }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\", \"bar\", \"onUpdate:bar\"]))\n}"
    );
}

#[test]
fn entries_carry_their_own_modifiers() {
    // The corpus row `directives/v_models_mods`. Vize emits the `<arg>Modifiers`
    // object after the update handler where babel emits it before; that literal
    // ordering difference is the one already accepted for single `v-model` on a
    // component (`directives/v_model_component_arg_mods`).
    assert_eq!(
        render_code("const A = () => <B v-models={[[foo, ['m']], [bar, 'bar', ['m']]]}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, {\n    \
         modelValue: foo,\n    \
         \"onUpdate:modelValue\": $event => ((foo) = $event),\n    \
         modelModifiers: { m: true },\n    \
         bar: bar,\n    \
         \"onUpdate:bar\": $event => ((bar) = $event),\n    \
         barModifiers: { m: true }\n  \
         }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\", \"bar\", \"onUpdate:bar\"]))\n}"
    );
}

#[test]
fn an_entry_combines_a_member_target_an_argument_and_modifiers() {
    assert_eq!(
        render_code("const A = () => <B v-models={[[a.b, 'x', ['trim', 'lazy']]]}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, {\n    \
         x: a.b,\n    \
         \"onUpdate:x\": $event => ((a.b) = $event),\n    \
         xModifiers: { trim: true, lazy: true }\n  \
         }, null, 8 /* PROPS */, [\"x\", \"onUpdate:x\"]))\n}"
    );
}

#[test]
fn every_entry_lowers_to_its_own_model_directive() {
    // The IR shape behind the codegen above: one `model` directive per entry,
    // never a single `models` directive.
    let bump = Allocator::new();
    let root = lower_one(
        &bump,
        "const A = () => <B v-models={[[foo], [bar, 'bar', ['trim']]]}/>;",
    );
    let element = root_element(&root);
    assert!(find_directive(element, "models").is_none());
    assert_eq!(element.props.len(), 2);

    let first = common::as_directive(&element.props[0]);
    assert_eq!(first.name, "model");
    assert!(first.arg.is_none());
    assert_eq!(simple_content(first.exp.as_ref().unwrap()), "foo");
    assert_eq!(first.modifiers.len(), 0);

    let second = common::as_directive(&element.props[1]);
    assert_eq!(second.name, "model");
    assert_eq!(simple_content(second.arg.as_ref().unwrap()), "bar");
    assert_eq!(simple_content(second.exp.as_ref().unwrap()), "bar");
    assert_eq!(
        second
            .modifiers
            .iter()
            .map(|modifier| modifier.content)
            .collect::<Vec<_>>(),
        vec!["trim"]
    );
}

#[test]
fn a_non_array_value_is_rejected() {
    // Babel rejects this too (corpus row `errors/v_models_not_array`).
    let source = "const A = () => <B v-models={foo}/>;";
    assert_eq!(
        errors(source),
        vec![
            "v-models expects a two-dimensional array of `[value, arg?, modifiers?]` \
             entries, e.g. v-models={[[foo], [bar, \"bar\"]]}."
                .to_string()
        ]
    );
    // No prop is contributed, so no `resolveDirective("models")` is emitted.
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B))\n}"
    );
}

#[test]
fn a_missing_value_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-models/>;"),
        vec![
            "v-models expects a two-dimensional array of `[value, arg?, modifiers?]` \
             entries, e.g. v-models={[[foo], [bar, \"bar\"]]}."
                .to_string()
        ]
    );
}

#[test]
fn an_empty_array_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-models={[]}/>;"),
        vec![
            "v-models was given an empty array; it needs at least one \
             `[value, arg?, modifiers?]` entry."
                .to_string()
        ]
    );
}

#[test]
fn a_non_array_entry_is_rejected() {
    // Babel: "You should pass a Two-dimensional Arrays to v-models"
    // (corpus row `errors/v_models_entry_not_array`).
    assert_eq!(
        errors("const A = () => <B v-models={[foo]}/>;"),
        vec![
            "v-models entry `foo` is not an array; each entry must be \
             `[value, arg?, modifiers?]`."
                .to_string()
        ]
    );
    // A spread cannot be destructured into entries either.
    assert_eq!(
        errors("const A = () => <B v-models={[...pairs]}/>;"),
        vec![
            "v-models entry `...pairs` is not an array; each entry must be \
             `[value, arg?, modifiers?]`."
                .to_string()
        ]
    );
}

#[test]
fn an_empty_entry_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-models={[[]]}/>;"),
        vec![
            "v-models entry `[]` has no bound value; each entry must be \
             `[value, arg?, modifiers?]`."
                .to_string()
        ]
    );
}

#[test]
fn a_non_assignable_entry_target_is_rejected() {
    // Same reason `v-model` rejects one (#3420): the write-back compiles to
    // `target = $event`, so a call expression would emit code that cannot parse.
    let source = "const A = () => <B v-models={[[get()], [ok]]}/>;";
    assert_eq!(
        errors(source),
        vec![
            "v-models target `get()` cannot be assigned to; \
             v-model needs a variable or property reference."
                .to_string()
        ]
    );
    // One bad entry rejects the whole attribute: a build error paired with a
    // partial set of model bindings would be harder to reason about than none.
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B))\n}"
    );
}

#[test]
fn a_dynamic_entry_argument_rejects_the_whole_attribute() {
    let source = "const A = () => <B v-models={[[ok], [foo, bar]]}/>;";
    assert_eq!(
        errors(source),
        vec![
            "v-model argument `bar` must be a string literal; dynamic arguments are not supported."
                .to_string()
        ]
    );
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B))\n}"
    );
}

#[test]
fn the_argument_spelling_is_rejected() {
    // Babel accepts `v-models:x` but then ignores every entry's own argument:
    // `v-models:x={[[a], [b, "b"]]}` binds `x` and `modelValue`, never `b`.
    assert_eq!(
        errors("const A = () => <B v-models:x={[[foo]]}/>;"),
        vec![
            "v-models does not take an argument; name the prop inside each entry \
             instead, e.g. v-models={[[value, \"name\"]]}."
                .to_string()
        ]
    );
}

#[test]
fn the_underscore_modifier_spelling_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-models_lazy={[[foo]]}/>;"),
        vec![
            "v-models does not take `_`-suffixed modifiers; list them inside each \
             entry instead, e.g. v-models={[[value, [\"lazy\"]]]}."
                .to_string()
        ]
    );
}

#[test]
fn a_plain_element_is_rejected_but_a_custom_element_is_not() {
    // Babel: "v-models can only use in custom components".
    assert_eq!(
        errors("const A = () => <input v-models={[[foo]]}/>;"),
        vec![
            "v-models can only be used on a component; a plain element binds one \
             model with v-model."
                .to_string()
        ]
    );
    // A dashed lowercase tag is a custom element: Vize classifies it as an
    // intrinsic element but the DOM backend resolves it with `resolveComponent`,
    // so `v-models` is legitimate there and must not be rejected.
    let source = "const A = () => <my-el v-models={[[foo]]}/>;";
    assert_eq!(errors(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_my_el = _resolveComponent(\"my-el\")\n  \n  \
         return (_openBlock(), _createBlock(_component_my_el, {\n    \
         modelValue: foo,\n    \
         \"onUpdate:modelValue\": $event => ((foo) = $event)\n  \
         }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))\n}"
    );
}
