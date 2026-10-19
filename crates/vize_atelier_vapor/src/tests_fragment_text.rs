//! Text runs beside elements in a fragment (a root, a slot, a `<template>`
//! carrier) render one text node, as `@vue/compiler-vapor` 3.6.0-rc.11 does.
//! Split nodes cannot hydrate the single text node server markup carries.

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

#[test]
fn mixed_text_runs_beside_elements_render_one_text_node() {
    for retained in [false, true] {
        for (source, text) in [
            (
                "<span/>{{ a }}A{{ b }}B<span/>",
                r#"_toDisplayString(_ctx.a) + "A" + _toDisplayString(_ctx.b) + "B""#,
            ),
            (
                "<Comp>Total: {{ a }} items<b/></Comp>",
                r#""Total: " + _toDisplayString(_ctx.a) + " items""#,
            ),
            (
                r#"<ul><template v-for="i in l">{{ i }} - <li/></template></ul>"#,
                r#"_toDisplayString(_for_item0.value) + " - ""#,
            ),
            (
                r#"<template v-if="ok">Hi {{ a }}<i/></template>"#,
                r#""Hi " + _toDisplayString(_ctx.a)"#,
            ),
        ] {
            let code = compiled(source, retained);
            assert!(code.contains(text), "{source}\n{code}");
            assert_eq!(code.matches("_setText(").count(), 1, "{source}\n{code}");
        }
    }
}

#[test]
fn static_text_beside_elements_stays_static() {
    for retained in [false, true] {
        let code = compiled("<span/>plain<span/>", retained);
        assert!(!code.contains("_setText("), "{code}");
    }
}
