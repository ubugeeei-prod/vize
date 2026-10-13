//! A component that hydrates as Vapor keeps the anchors the Vapor runtime
//! claims in its server markup, as `@vue/compiler-ssr` emits with `vapor`.
#![expect(clippy::panic, reason = "tests assert by panicking")]

use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_l0::{Allocator, String};

fn compile(source: &str, vapor: bool) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_with_options(
        &allocator,
        source,
        SsrCompilerOptions {
            vapor,
            ..SsrCompilerOptions::default()
        },
    );
    if !errors.is_empty() {
        panic!("{source}: {errors:?}");
    }
    result.code
}

#[test]
fn transition_group_keeps_the_empty_branch_and_nested_fragments() {
    let source = r#"<TransitionGroup tag="ul"><li v-if="a" key="a">a</li><template v-for="i in l"><li :key="i">{{ i }}</li></template></TransitionGroup>"#;

    let plain = compile(source, false);
    assert!(!plain.contains("<!---->"), "{plain}");
    assert!(!plain.contains("<!--[-->"), "{plain}");

    let vapor = compile(source, true);
    assert!(vapor.contains("_push(`<!---->`)"), "{vapor}");
    let open = vapor.find("<!--[-->").unwrap_or(usize::MAX);
    let list = vapor.find("_ssrRenderList(").unwrap_or(usize::MAX);
    let close = vapor.find("<!--]-->").unwrap_or(0);
    let end = vapor.find("</ul>").unwrap_or(0);
    assert!(open < list && list < close && close < end, "{vapor}");
}

#[test]
fn each_suspense_slot_renders_as_one_fragment() {
    let source = r#"<Suspense><div>a</div><template #fallback>f</template></Suspense>"#;

    let plain = compile(source, false);
    assert!(plain.contains("_push(`<div>a</div>`)"), "{plain}");
    assert!(plain.contains("_push(`f`)"), "{plain}");

    let vapor = compile(source, true);
    assert!(
        vapor.contains("_push(`<!--[--><div>a</div><!--]-->`)"),
        "{vapor}"
    );
    assert!(vapor.contains("_push(`<!--[-->f<!--]-->`)"), "{vapor}");
}

#[test]
fn a_lone_suspense_child_is_wrapped_too() {
    let source = "<Suspense><Comp/></Suspense>";

    let plain = compile(source, false);
    assert!(!plain.contains("<!--[-->"), "{plain}");

    let vapor = compile(source, true);
    let open = vapor.find("_push(`<!--[-->`)").unwrap_or(usize::MAX);
    let component = vapor.find("_ssrRenderComponent(").unwrap_or(usize::MAX);
    let close = vapor.find("_push(`<!--]-->`)").unwrap_or(0);
    assert!(open < component && component < close, "{vapor}");
}

#[test]
fn markup_outside_those_boundaries_is_unchanged() {
    for source in [
        r#"<div><p v-if="a">a</p><span v-for="i in l" :key="i">{{ i }}</span></div>"#,
        r#"<Comp><template #x="{ a }">{{ a }}</template></Comp>"#,
        r#"<template v-if="a"><b></b><i></i></template>"#,
    ] {
        assert_eq!(compile(source, false), compile(source, true), "{source}");
    }
}
