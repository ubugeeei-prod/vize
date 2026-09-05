//! `v-slots` lowering (#3418).
//!
//! `v-slots` is a `@vue/babel-plugin-jsx` built-in, not a user directive. Before
//! this was implemented it fell through the generic custom-directive path and
//! compiled to `resolveDirective("slots")` — a lookup Vue resolves to nothing at
//! runtime, so the component rendered with no slots, no error and no warning. For
//! an object-literal value it was worse: the directive value was emitted as the
//! **raw attribute source**, so `v-slots={{ default: () => <i/> }}` put unparsed
//! JSX into the generated JavaScript module.
//!
//! The object-literal form is pinned against the real plugin by the differential
//! oracle (rows `slots/v_slots_object_literal` and
//! `slots/v_slots_object_with_children`). Forwarding an **opaque** slots object
//! (`v-slots={slots}`, #3467) has its own suite in `v_slots_forwarding.rs`;
//! what stays here is the object-literal path plus the shapes both files agree
//! are not a slots object at all.

mod common;

use vize_atelier_jsx::{JsxLang, VdomCompileOptions, compile_to_vdom, lower_source};
use vize_s0::Allocator;

fn diagnostics(source: &str) -> Vec<String> {
    let bump = Allocator::new();
    let out = lower_source(&bump, bump.as_oxc(), source, JsxLang::Jsx);
    out.diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str().to_string())
        .collect()
}

fn errors(source: &str) -> Vec<String> {
    let bump = Allocator::new();
    let out = lower_source(&bump, bump.as_oxc(), source, JsxLang::Jsx);
    out.diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .map(|diagnostic| diagnostic.message.as_str().to_string())
        .collect()
}

/// Errors from the whole VDOM compile, so checks the shared transform makes
/// (rather than JSX lowering) are visible too.
fn compile_errors(source: &str) -> Vec<String> {
    let bump = Allocator::new();
    let out = compile_to_vdom(&bump, source, JsxLang::Jsx, VdomCompileOptions::default());
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

/// The rejected shapes all leave a component with no slots and, crucially, no
/// `resolveDirective("slots")`.
const BARE_COMPONENT: &str = "export function render(_ctx, _cache) {\n  \
     const _component_B = _resolveComponent(\"B\")\n  \n  \
     return (_openBlock(), _createBlock(_component_B))\n}";

#[test]
fn a_default_slot_object_becomes_the_default_slot() {
    let source = "const A = () => <B v-slots={{default: () => <i/>}}/>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"i\")\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn named_slots_each_become_their_own_slot() {
    // The corpus row `slots/v_slots_object_literal`: babel emits
    // `{default: () => createVNode("i"), bar: () => createVNode("b")}`.
    let source = "const A = () => <B v-slots={{default: () => <i/>, bar: () => <b/>}}/>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"i\")\n    \
         ]),\n    \
         bar: _withCtx(() => [\n      \
         _createElementVNode(\"b\")\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn a_scoped_slot_keeps_its_params() {
    let source = "const A = () => <B v-slots={{header: (p) => <i>{p.x}</i>}}/>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         header: _withCtx((p) => [\n      \
         _createElementVNode(\"i\", null, _toDisplayString(p.x), 1 /* TEXT */)\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn element_children_still_become_the_default_slot() {
    // The corpus row `slots/v_slots_object_with_children`: babel emits
    // `{default: () => [<div>A</div>], foo: () => <b/>}`. Vize emits the same two
    // slots in the other literal order.
    let source = "const A = () => <B v-slots={{foo: () => <b/>}}><div>A</div></B>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         foo: _withCtx(() => [\n      \
         _createElementVNode(\"b\")\n    \
         ]),\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"div\", null, \"A\")\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn a_default_slot_plus_children_is_diagnosed_not_silently_resolved() {
    // Babel emits the `default` key twice and lets JavaScript keep the later
    // one, silently discarding the children. Vize names the ambiguity instead,
    // via the shared transform's existing check.
    assert_eq!(
        compile_errors("const A = () => <B v-slots={{default: () => <i/>}}><div>A</div></B>;"),
        vec![
            "Extraneous children found when component already has an explicit default slot."
                .to_string()
        ]
    );
}

#[test]
fn a_string_slot_name_is_kept_verbatim() {
    let source = "const A = () => <B v-slots={{'my-slot': () => <i/>}}/>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(
        render_code(source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         \"my-slot\": _withCtx(() => [\n      \
         _createElementVNode(\"i\")\n    \
         ]),\n    \
         _: 1 /* STABLE */\n  \
         }))\n}"
    );
}

#[test]
fn an_empty_slots_object_contributes_nothing() {
    let source = "const A = () => <B v-slots={{}}/>;";
    assert_eq!(diagnostics(source), Vec::<String>::new());
    assert_eq!(render_code(source), BARE_COMPONENT);
}

#[test]
fn v_slots_is_never_a_prop() {
    // The regression itself: no `slots` directive survives lowering, so no
    // `resolveDirective("slots")` and no raw JSX source can reach the output.
    let bump = Allocator::new();
    let root = common::lower_one(
        &bump,
        "const A = () => <B class=\"c\" v-slots={{foo: () => <b/>}}/>;",
    );
    let element = common::root_element(&root);
    assert!(common::find_directive(element, "slots").is_none());
    assert_eq!(element.props.len(), 1);
    assert_eq!(common::as_attribute(&element.props[0]).name, "class");
}

#[test]
fn a_literal_value_is_rejected() {
    // The corpus row `errors/v_slots_not_object`: babel forwards `1` as the
    // component's children, which is meaningless. An opaque *expression* is
    // forwarded instead of rejected — see `v_slots_forwarding.rs` (#3467).
    let suffix = "Write the slots inline, e.g. v-slots={{ default: () => <div/> }}, or \
                  forward a slots object, e.g. v-slots={slots}.";
    let not_slots = "is not a slots object: babel forwards it as the component's children, \
                     which leaves the component with no slots.";
    assert_eq!(
        errors("const A = () => <B v-slots={1}/>;"),
        vec![format!("v-slots value `1` {not_slots} {suffix}")]
    );
    assert_eq!(
        errors("const A = () => <B v-slots=\"str\"/>;"),
        vec![format!("v-slots value `\"str\"` {not_slots} {suffix}")]
    );
    assert_eq!(
        errors("const A = () => <B v-slots={[a, b]}/>;"),
        vec![format!("v-slots value `[a, b]` {not_slots} {suffix}")]
    );
    assert_eq!(
        render_code("const A = () => <B v-slots={1}/>;"),
        BARE_COMPONENT
    );
    // A lone function is the default slot, not a slots object: spreading it
    // would contribute nothing, so it is named rather than mis-lowered.
    assert_eq!(
        errors("const A = () => <B v-slots={() => <i/>}/>;"),
        vec![format!(
            "v-slots value `() => <i/>` is a function, not a slots object: a lone function \
             is the default slot, so a spread of it contributes nothing. {suffix}"
        )]
    );
}

#[test]
fn a_missing_value_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-slots/>;"),
        vec![
            "v-slots is missing its slots object, e.g. v-slots={{ default: () => <div/> }}."
                .to_string()
        ]
    );
    assert_eq!(render_code("const A = () => <B v-slots/>;"), BARE_COMPONENT);
}

#[test]
fn the_argument_spelling_is_rejected() {
    assert_eq!(
        errors("const A = () => <B v-slots:x={{foo: () => <i/>}}/>;"),
        vec![
            "v-slots does not take an argument; the slot names are the keys of its \
             object, e.g. v-slots={{ header: () => <h1/> }}."
                .to_string()
        ]
    );
}

#[test]
fn a_plain_element_is_rejected() {
    // Babel drops `v-slots` on a non-component silently, emitting `[]` children.
    assert_eq!(
        errors("const A = () => <div v-slots={{foo: () => <i/>}}/>;"),
        vec!["v-slots can only be used on a component; a plain element has no slots.".to_string()]
    );
    assert_eq!(
        render_code("const A = () => <div v-slots={{foo: () => <i/>}}/>;"),
        "export function render(_ctx, _cache) {\n  \
         return (_openBlock(), _createElementBlock(\"div\"))\n}"
    );
}

#[test]
fn a_repeated_v_slots_is_rejected() {
    // Babel keeps the last one and drops the rest silently.
    assert_eq!(
        errors("const A = () => <B v-slots={{a: () => <i/>}} v-slots={{b: () => <i/>}}/>;"),
        vec![
            "v-slots is given more than once on this element; babel keeps only the last \
             one, so merge them into a single slots object."
                .to_string()
        ]
    );
}

#[test]
fn spreads_keep_their_object_children_warning() {
    // `lower_object_slots` is shared with the object-children idiom, so spread
    // entries stay warnings naming what was ignored rather than silence.
    assert_eq!(
        diagnostics("const A = () => <B v-slots={{...rest}}/>;"),
        vec!["spread in a JSX slot object is not supported and was ignored".to_string()]
    );
    let dynamic_source = "const π = n; const A = () => <B v-slots={{[π]: () => <i/>}}/>;";
    assert_eq!(diagnostics(dynamic_source), Vec::<String>::new());
    assert_eq!(
        render_code(dynamic_source),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         [π]: _withCtx(() => [\n      \
         _createElementVNode(\"i\")\n    \
         ]),\n    \
         _: 2 /* DYNAMIC */\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}
