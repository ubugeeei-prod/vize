//! #11593: a slot's VNode fallback merges adjacent text and interpolations
//! into one text child, as Vue's `transformText` does. Nuxt's `<Title>` reads
//! its slot in VNode mode and accepts exactly one child.

use vize_atelier_ssr::compile_ssr;
use vize_l0::{Allocator, String};

fn compile(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, source);
    assert!(errors.is_empty(), "{source}: {errors:?}");
    result.code
}

/// The VNode fallback of the first slot: the `return [...]` line of its `else`.
fn vnode_fallback(code: &str) -> &str {
    let start = code.find("return [").unwrap_or(code.len());
    let rest = code.get(start..).unwrap_or_default();
    rest.split('\n').next().unwrap_or_default()
}

const SLOT_RUNS: &[(&str, &str)] = &[
    (
        "<Title>{{ count }} Foo</Title>",
        r#"return [_createTextVNode(_toDisplayString(_ctx.count) + " Foo")]"#,
    ),
    (
        "<Title>Foo {{ count }}</Title>",
        r#"return [_createTextVNode("Foo " + _toDisplayString(_ctx.count))]"#,
    ),
    (
        "<Title>{{ count }} {{ name }}</Title>",
        r#"return [_createTextVNode(_toDisplayString(_ctx.count) + " " + _toDisplayString(_ctx.name))]"#,
    ),
    (
        "<Title>{{ count }}{{ name }}</Title>",
        r#"return [_createTextVNode(_toDisplayString(_ctx.count) + _toDisplayString(_ctx.name))]"#,
    ),
    // Negative controls: a lone text or interpolation was already one child.
    ("<Title>Foo</Title>", r#"return [_createTextVNode("Foo")]"#),
    (
        "<Title>{{ count }}</Title>",
        r#"return [_createTextVNode(_toDisplayString(_ctx.count))]"#,
    ),
];

const MIXED: &str = "<Comp>x {{ b }}<i></i>y<div>{{ a }} b</div><p>{{ a }}</p></Comp>";
const MIXED_FALLBACK: &str = r#"return [_createTextVNode("x " + _toDisplayString(_ctx.b)), _createElementVNode("i", null, null), _createTextVNode("y"), _createElementVNode("div", null, [_createTextVNode(_toDisplayString(_ctx.a) + " b")]), _createElementVNode("p", null, [_createTextVNode(_toDisplayString(_ctx.a))])]"#;

const NESTED: &[(&str, &str)] = &[
    (
        r#"<Comp><template v-if="ok">{{ a }} b</template></Comp>"#,
        r#"return [(_ctx.ok) ? _createVNode(_Fragment, null, [_createTextVNode(_toDisplayString(_ctx.a) + " b")]) : _createCommentVNode("", true)]"#,
    ),
    (
        r#"<Comp><span v-for="i in list" :key="i">{{ i }} x</span></Comp>"#,
        r#"return [(_openBlock(true), _createBlock(_Fragment, null, _renderList(_ctx.list, (i) => { return _createElementVNode("span", { key: i }, [_createTextVNode(_toDisplayString(i) + " x")]) }), 128 /* KEYED_FRAGMENT */))]"#,
    ),
    (
        "<Comp><slot>{{ a }} b</slot></Comp>",
        r#"return [_renderSlot(_ctx.$slots, "default", {}, () => [_createTextVNode(_toDisplayString(_ctx.a) + " b")])]"#,
    ),
];

#[test]
fn a_slot_text_run_is_one_text_vnode() {
    for (source, expected) in SLOT_RUNS {
        assert_eq!(vnode_fallback(&compile(source)), *expected, "{source}");
    }
}

#[test]
fn elements_end_a_text_run_and_their_own_children_merge() {
    assert_eq!(vnode_fallback(&compile(MIXED)), MIXED_FALLBACK);
}

#[test]
fn branch_loop_and_outlet_fallback_runs_merge() {
    for (source, expected) in NESTED {
        assert_eq!(vnode_fallback(&compile(source)), *expected, "{source}");
    }
}

/// The L4 lane and the legacy walker emit the same bytes for every case.
#[cfg(feature = "legacy-differential")]
#[test]
fn both_lanes_emit_the_merged_runs() {
    use vize_atelier_ssr::differential::compare_ssr_lanes;
    use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

    let sources = SLOT_RUNS
        .iter()
        .chain(NESTED)
        .map(|(source, _)| *source)
        .chain([MIXED]);
    for source in sources {
        let pair = compare_ssr_lanes(
            source,
            &SsrCompilerOptions::default(),
            &SsrCompilerExperimentalOptions::default(),
        );
        assert!(pair.legacy.errors.is_empty(), "{source}");
        assert_eq!(
            pair.selected.result.code, pair.legacy.result.code,
            "{source} ({})",
            pair.lane
        );
    }
}
