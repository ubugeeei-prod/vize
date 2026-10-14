//! Several delegated listeners for one event on one element all stay bound.
//! The first owns the element's `$evt` slot; assigning it again would drop
//! that listener, so each later one joins it through `delegate`.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn compile(source: &str, davinci_retained_lane: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code
}

#[test]
fn a_second_listener_for_the_same_event_joins_the_first() {
    let source = r#"<div @click.delegate="first" @click.delegate.prevent="second"></div>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert_eq!(code.matches("n0.$evtclick = ").count(), 1, "{code}");
        assert!(
            code.contains("n0.$evtclick = _createInvoker(e => _ctx.first(e))"),
            "{code}"
        );
        assert!(
            code.contains(
                r#"_delegate(n0, "click", _withModifiers(e => _ctx.second(e), ["prevent"]))"#
            ),
            "{code}"
        );
        assert!(code.contains("delegate as _delegate"), "{code}");
        assert!(code.contains(r#"_delegateEvents("click")"#), "{code}");
    }
}

#[test]
fn separate_elements_and_events_keep_their_own_slots() {
    let source =
        r#"<div @click.delegate="a" @input.delegate="b"><span @click.delegate="c"></span></div>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert_eq!(code.matches(".$evtclick = ").count(), 2, "{code}");
        assert_eq!(code.matches(".$evtinput = ").count(), 1, "{code}");
        assert!(!code.contains("_delegate("), "{code}");
    }
}

#[test]
fn each_loop_item_assigns_its_own_first_listener() {
    let source = r#"<li v-for="item in items" @click.delegate="pick(item)" @click.delegate="log(item)"></li>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert_eq!(code.matches(".$evtclick = ").count(), 1, "{code}");
        assert_eq!(
            code.matches(r#"_delegate(n2, "click", "#).count(),
            1,
            "{code}"
        );
    }
}

#[test]
fn a_stop_listener_for_the_same_event_keeps_both_listeners_direct() {
    let source = r#"<div @click.delegate="first" @click.stop="second"></div>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(!code.contains("$evtclick"), "{code}");
        assert!(
            code.contains(r#"_on(n0, "click", e => _ctx.first(e))"#),
            "{code}"
        );
        assert!(
            code.contains(r#"_on(n0, "click", _withModifiers(e => _ctx.second(e), ["stop"]))"#),
            "{code}"
        );
    }
}
