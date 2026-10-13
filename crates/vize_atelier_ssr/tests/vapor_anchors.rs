//! A component that hydrates as Vapor keeps the anchors the Vapor runtime
//! claims in its server markup, as `@vue/compiler-ssr` emits with `vapor`.
//!
//! Each shape is pinned whole, with and without the option, so the anchors
//! are the only difference a reader sees between the paired snapshots.
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

const TRANSITION_GROUP: &str = r#"<TransitionGroup tag="ul"><li v-if="a" key="a">a</li><template v-for="i in l"><li :key="i">{{ i }}</li></template></TransitionGroup>"#;
const SUSPENSE_SLOTS: &str = "<Suspense><div>a</div><template #fallback>f</template></Suspense>";
const SUSPENSE_CHILD: &str = "<Suspense><Comp/></Suspense>";

#[test]
fn transition_group_keeps_the_empty_branch_and_nested_fragments() {
    insta::assert_snapshot!("transition_group_plain", compile(TRANSITION_GROUP, false));
    insta::assert_snapshot!("transition_group_vapor", compile(TRANSITION_GROUP, true));
}

#[test]
fn each_suspense_slot_renders_as_one_fragment() {
    insta::assert_snapshot!("suspense_slots_plain", compile(SUSPENSE_SLOTS, false));
    insta::assert_snapshot!("suspense_slots_vapor", compile(SUSPENSE_SLOTS, true));
}

#[test]
fn a_lone_suspense_child_is_wrapped_too() {
    insta::assert_snapshot!("suspense_child_plain", compile(SUSPENSE_CHILD, false));
    insta::assert_snapshot!("suspense_child_vapor", compile(SUSPENSE_CHILD, true));
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
