//! Vue 3.6 `setInsertionState` model, pinned against `@vue/compiler-vapor`
//! 3.6.0-rc.6 output for the same templates.
//!
//! A block that needs insertion (component, slot outlet, `v-if`, `v-for`)
//! keeps a template placeholder only when a template-rendered sibling follows
//! it, and is then inserted before that placeholder. Trailing blocks have no
//! placeholder: they append, carrying their hydration start unit (the number
//! of logical units before them, omitted when zero). Both lowering lanes must
//! agree, since hydration walks the server markup by these units.

#![expect(clippy::disallowed_types, reason = "test fixtures compare std strings")]

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compiled(source: &str, retained: bool) -> std::string::String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            davinci_retained_lane: retained,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    std::string::String::from(result.code.as_str())
}

/// Template strings and the render body without indentation or imports.
fn body(code: &str) -> std::string::String {
    code.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("import "))
        .collect::<std::vec::Vec<_>>()
        .join("\n")
}

/// Runs `check` against the selected and the retained lane; hydration walks
/// the server markup by the same units in both, so both must agree.
fn each_lane(source: &str, check: impl Fn(&str)) {
    for retained in [false, true] {
        check(&body(&compiled(source, retained)));
    }
}

/// Exact output, identical in both lanes.
fn shape(source: &str) -> std::string::String {
    let selected = body(&compiled(source, false));
    assert_eq!(
        selected,
        body(&compiled(source, true)),
        "lanes disagree for {source}"
    );
    selected
}

#[test]
fn a_sole_trailing_slot_appends_without_a_placeholder() {
    assert_eq!(
        shape("<div><slot/></div>"),
        r#"const t0 = _template("<div></div>", true)
export function render(_ctx) {
const n1 = t0()
_setInsertionState(n1)
const n0 = _createSlot("default")
return n1
}"#
    );
}

#[test]
fn a_block_before_rendered_siblings_is_inserted_before_its_placeholder() {
    assert_eq!(
        shape("<div><span/><slot/><span/></div>"),
        r#"const t0 = _template("<div><span></span><!----><span></span></div>", true)
export function render(_ctx) {
const n1 = t0()
const n2 = _next(_child(n1), 1)
_setInsertionState(n1, n2)
const n0 = _createSlot("default")
return n1
}"#
    );
}

#[test]
fn trailing_blocks_carry_their_hydration_unit_index() {
    // Upstream: `_setInsertionState(n2, 2)` then `(n2, 3)`: a text run and an
    // element precede the first component, and it precedes the second.
    each_lane("<div>hi <b>x</b><Comp/><Comp/></div>", |code| {
        assert!(
            code.contains(r#"_template("<div>hi <b>x</b></div>", true)"#),
            "{code}"
        );
        let states = insertion_states(code);
        assert_eq!(states.len(), 2, "{code}");
        assert!(states[0].ends_with(", 2)"), "{code}");
        assert!(states[1].ends_with(", 3)"), "{code}");
        assert_eq!(
            code.matches("_createComponentWithFallback(").count(),
            2,
            "{code}"
        );
    });

    // A trailing slot after a `v-if` block counts the block as one unit;
    // upstream: `_setInsertionState(n4)` then `_setInsertionState(n4, 1)`.
    each_lane(r#"<div><p v-if="x">a</p><slot/></div>"#, |code| {
        assert!(code.contains(r#"_template("<div></div>", true)"#), "{code}");
        let states = insertion_states(code);
        assert_eq!(states.len(), 2, "{code}");
        assert!(!states[0].contains(','), "{code}");
        assert!(states[1].ends_with(", 1)"), "{code}");
    });
}

#[test]
fn interpolation_runs_are_one_unit_and_keep_placeholders_only_before_text() {
    // Upstream: `<div> <!> ` with the slot inserted before its placeholder.
    each_lane("<div>{{ a }}<slot/>{{ b }}</div>", |code| {
        assert!(
            code.contains(r#"_template("<div> <!----> </div>", true)"#),
            "{code}"
        );
        let states = insertion_states(code);
        assert_eq!(states.len(), 1, "{code}");
        assert!(states[0].contains(", n"), "{code}");
    });

    // Upstream: `<div> ` and `_setInsertionState(n2, 1)`.
    each_lane("<div>{{ a }}<slot/></div>", |code| {
        assert!(
            code.contains(r#"_template("<div> </div>", true)"#),
            "{code}"
        );
        let states = insertion_states(code);
        assert_eq!(states.len(), 1, "{code}");
        assert!(states[0].ends_with(", 1)"), "{code}");
    });
}

#[test]
fn branch_and_loop_roots_never_repeat_the_parent_insertion_state() {
    each_lane(
        r#"<main><Comp v-if="tab"/><i v-for="x in xs">{{ x }}</i></main>"#,
        |code| {
            let states = insertion_states(code);
            assert_eq!(states.len(), 2, "{code}");
            assert!(!states[0].contains(','), "{code}");
            assert!(states[1].ends_with(", 1)"), "{code}");
        },
    );
}

#[test]
fn slot_outlets_use_insertion_state_instead_of_insert() {
    for retained in [false, true] {
        let code = compiled(
            "<ul><li>a</li><slot name=\"tail\" :x=\"x\"/></ul>",
            retained,
        );
        assert!(!code.contains("insert as _insert"), "{code}");
        let states = insertion_states(&code);
        assert_eq!(states.len(), 1, "{code}");
        assert!(states[0].ends_with(", 1)"), "{code}");
    }
}

#[test]
fn dynamic_event_names_rebind_through_on_binding() {
    // Upstream: `_onBinding(n0, _ctx.ev, handler, { once: true })` inside a
    // render effect, so the previous listener is removed on every rename.
    let code = compiled(r#"<button @[ev].once="go">x</button>"#, false);
    assert!(code.contains("onBinding as _onBinding"), "{code}");
    assert!(
        code.contains("_onBinding(n0, _ctx.ev, e => _ctx.go(e), {"),
        "{code}"
    );
    assert!(code.contains("once: true"), "{code}");
    assert!(!code.contains("effect: true"), "{code}");
    assert!(!code.contains("createInvoker"), "{code}");
}

fn insertion_states(code: &str) -> std::vec::Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("_setInsertionState("))
        .collect()
}
