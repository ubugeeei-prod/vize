//! `v-bind` modifiers on a component prop choose how the value lands when the
//! props fall through to an element: `.prop` as a DOM property, `.attr` as an
//! attribute and `.camel` under the camelized name.

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
fn static_keys_carry_their_modifier() {
    let source = r#"<Child :a.attr="b" :c-d.camel="e" :f-g.prop="h" .i="j" :k="l"/>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(code.contains(r#""^a": () => (_ctx.b)"#), "{code}");
        assert!(code.contains("cD: () => (_ctx.e)"), "{code}");
        assert!(code.contains(r#"".f-g": () => (_ctx.h)"#), "{code}");
        assert!(code.contains(r#"".i": () => (_ctx.j)"#), "{code}");
        assert!(code.contains("k: () => (_ctx.l)"), "{code}");
    }
}

#[test]
fn dynamic_keys_are_prefixed_at_run_time() {
    let source = r#"<Child :[k].prop="v" :[m].attr="w"/>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains(r#"[("." + (_ctx.k || ""))]: _ctx.v"#),
            "{code}"
        );
        assert!(
            code.contains(r#"[("^" + (_ctx.m || ""))]: _ctx.w"#),
            "{code}"
        );
    }
}
