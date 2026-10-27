//! Regression for naitokosuke's #10894: component listeners retain guards and options.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn compile(source: &str, retained: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane: retained,
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
fn component_event_modifiers_guard_handlers_and_suffix_listener_options() {
    for retained in [false, true] {
        let code = compile(
            r#"<Box @submit.prevent="submit" @keydown.enter.stop="key" @click.right.once.capture.passive="click" /><Pinger @ping.once="ping" />"#,
            retained,
        );
        for expected in [
            r#"onSubmit: () => (_withModifiers(_ctx.submit, ["prevent"]))"#,
            r#"onKeydown: () => (_withKeys(_withModifiers(_ctx.key, ["stop"]), ["enter"]))"#,
            r#"onContextmenuOnceCapturePassive: () => (_withModifiers(_ctx.click, ["right"]))"#,
            "onPingOnce: () => (_ctx.ping)",
        ] {
            assert!(code.contains(expected), "missing {expected}: {code}");
        }
        assert!(code.contains("withModifiers as _withModifiers"), "{code}");
        assert!(code.contains("withKeys as _withKeys"), "{code}");
    }
}

#[test]
fn component_event_modifiers_keep_dynamic_keys_inline_handlers_and_empty_handlers() {
    for retained in [false, true] {
        let code = compile(r#"<Box @[event].enter.prevent.once="count++" />"#, retained);
        assert!(
            code.contains(r#"_toHandlerKey(_ctx.event) + "Once""#),
            "{code}"
        );
        assert!(code.contains("_withKeys(_withModifiers("), "{code}");
        assert!(code.contains("_ctx.count++"), "{code}");
        assert!(code.contains(r#"["prevent"]), ["enter"])"#), "{code}");
        let code = compile(r#"<Box @submit.prevent />"#, retained);
        assert!(
            code.contains(r#"_withModifiers(() => {}, ["prevent"])"#),
            "{code}"
        );
    }
}

#[test]
fn component_event_modifiers_merge_independently_guarded_same_event_handlers() {
    for retained in [false, true] {
        let code = compile(
            r#"<Foo @keydown.enter.prevent="a" @keydown.esc.prevent="b" @keydown.once="c" />"#,
            retained,
        );
        assert_eq!(code.matches("onKeydown:").count(), 1, "{code}");
        assert!(code.contains(r#"onKeydown: () => ([_withKeys(_withModifiers(_ctx.a, ["prevent"]), ["enter"]), _withKeys(_withModifiers(_ctx.b, ["prevent"]), ["esc"])])"#), "{code}");
        assert!(code.contains("onKeydownOnce: () => (_ctx.c)"), "{code}");
    }
}
