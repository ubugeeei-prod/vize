//! Regression tests for duplicate component event properties.

use vize_atelier_ssr::compile_ssr;
use vize_s0::{Allocator, String};

fn compile(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, src);
    assert!(errors.is_empty(), "Compilation errors: {errors:?}");
    result.code
}

#[test]
fn component_v_model_and_explicit_update_listener_are_merged() {
    let model_first = compile(r#"<Foo v-model="value" @update:modelValue="onUpdate" />"#);

    assert_eq!(
        model_first.matches(r#""onUpdate:modelValue":"#).count(),
        1,
        "{model_first}",
    );
    assert!(
        model_first.contains(
            r#"modelValue: _ctx.value, "onUpdate:modelValue": [$event => ((_ctx.value) = $event), _ctx.onUpdate]"#,
        ),
        "{model_first}",
    );

    let listener_first = compile(r#"<Foo @update:modelValue="onUpdate" v-model="value" />"#);
    assert_eq!(
        listener_first.matches(r#""onUpdate:modelValue":"#).count(),
        1,
        "{listener_first}",
    );
    assert!(
        listener_first.contains(
            r#""onUpdate:modelValue": [_ctx.onUpdate, $event => ((_ctx.value) = $event)], modelValue: _ctx.value"#,
        ),
        "{listener_first}",
    );
}

#[test]
fn component_modifier_handlers_share_one_runtime_event_prop() {
    let code = compile(r#"<Foo @keydown="a" @keydown.enter.prevent="b" />"#);

    assert_eq!(code.matches("onKeydown:").count(), 1, "{code}");
    assert!(code.contains("onKeydown: [_ctx.a, _ctx.b]"), "{code}");
}

#[test]
fn component_event_handlers_preserve_object_spread_boundaries() {
    let code = compile(r#"<Foo @click="before" v-bind="props" @click.stop="after" />"#);

    assert!(
        code.contains(
            r#"_mergeProps({ onClick: _ctx.before }, _normalizeProps(_guardReactiveProps(_ctx.props)), { onClick: _ctx.after })"#,
        ),
        "{code}",
    );
}

#[test]
fn component_event_handlers_preserve_dynamic_key_boundaries() {
    let code = compile(r#"<Foo @click="before" @[name]="middle" @click.stop="after" />"#);

    assert!(
        code.contains(
            r#"_mergeProps({ onClick: _ctx.before }, _normalizeProps({ [_toHandlerKey(_ctx.name) || ""]: _ctx.middle }), { onClick: _ctx.after })"#,
        ),
        "{code}",
    );
}
