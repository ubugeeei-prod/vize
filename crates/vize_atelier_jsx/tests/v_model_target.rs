//! `v-model` write-back target validation (#3420).
//!
//! `v-model` compiles to `$event => (target = $event)`, so its target has to be
//! a place expression. Before this was checked, a non-assignable target such as
//! `v-model={{a:1}}` lowered anyway and emitted
//! `$event => ($event => ($event => (({a:1}) = $event)))` — a module that does
//! not parse. `@vue/babel-plugin-jsx` rejects the same inputs, so diagnosing
//! here is both the correct behavior and the compatible one.

mod common;

use vize_atelier_jsx::{JsxLang, VdomCompileOptions, compile_to_vdom, lower_source};
use vize_s0::Allocator;

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
fn object_literal_target_is_rejected() {
    assert_eq!(
        errors("const A = () => <input v-model={{a:1}}/>;"),
        vec![
            "v-model target `{a:1}` cannot be assigned to; \
             v-model needs a variable or property reference."
                .to_string()
        ]
    );
    // The attribute contributes no prop, so none of the previously-emitted
    // invalid assignment code reaches the output.
    assert_eq!(
        render_code("const A = () => <input v-model={{a:1}}/>;"),
        "export function render(_ctx, _cache) {\n  return (_openBlock(), _createElementBlock(\"input\"))\n}"
    );
}

#[test]
fn call_and_literal_targets_are_rejected() {
    assert_eq!(
        errors("const A = () => <input v-model={get()}/>;"),
        vec![
            "v-model target `get()` cannot be assigned to; \
             v-model needs a variable or property reference."
                .to_string()
        ]
    );
    assert_eq!(
        errors("const A = () => <input v-model={\"str\"}/>;"),
        vec![
            "v-model target `\"str\"` cannot be assigned to; \
             v-model needs a variable or property reference."
                .to_string()
        ]
    );
}

#[test]
fn array_form_validates_only_the_first_element() {
    // `['trim']` is the modifiers list, not an assignment target, so the array
    // form must not be rejected for it.
    assert_eq!(
        errors("const A = () => <input v-model={[val, ['trim']]}/>;"),
        Vec::<String>::new()
    );
    // …but a non-assignable *first* element still is.
    assert_eq!(
        errors("const A = () => <input v-model={[{a:1}, ['trim']]}/>;"),
        vec![
            "v-model target `{a:1}` cannot be assigned to; \
             v-model needs a variable or property reference."
                .to_string()
        ]
    );
}

#[test]
fn assignable_targets_are_accepted() {
    // Identifiers, member access of both forms, optional chaining, and the
    // type-only wrappers TSX allows must all still lower cleanly.
    for source in [
        "const A = () => <input v-model={val}/>;",
        "const A = () => <input v-model={state.val}/>;",
        "const A = () => <input v-model={state[key]}/>;",
        "const A = () => <input v-model={state?.val}/>;",
        "const A = () => <input v-model={(val)}/>;",
        "const A = () => <B v-model={[val, 'arg', ['trim']]}/>;",
        "const A = () => <input v-model_lazy={val}/>;",
    ] {
        assert_eq!(errors(source), Vec::<String>::new(), "source: {source}");
    }
}

#[test]
fn missing_expression_keeps_its_own_diagnostic() {
    // `v-model` with no value at all is a different defect, already reported by
    // the core transform; the new check must not shadow or duplicate it.
    let bump = Allocator::new();
    let out = compile_to_vdom(
        &bump,
        "const A = () => <input v-model/>;",
        JsxLang::Jsx,
        VdomCompileOptions::default(),
    );
    let messages: Vec<String> = out
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str().to_string())
        .collect();
    assert_eq!(messages, vec!["v-model is missing expression.".to_string()]);
}
