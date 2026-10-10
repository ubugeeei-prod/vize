//! A created component's last argument marks it as its render's single root,
//! which alone receives the owner's fallthrough attrs. Vue's compiler sets it
//! only for the lone root, including through a root `v-if`, a keyed fragment or
//! a Transition, and never for a component nested in an element, among sibling
//! roots or inside `v-for`.

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

fn root_flags(code: &str) -> usize {
    code.matches(", null, true)").count()
}

#[test]
fn only_a_single_root_component_receives_the_owners_fallthrough_attrs() {
    for retained in [false, true] {
        let root = compile(r#"<Child :a="a" />"#, retained);
        assert_eq!(root_flags(&root), 1, "{root}");

        let nested = compile(r#"<section><Child :a="a" /></section>"#, retained);
        assert_eq!(root_flags(&nested), 0, "{nested}");
        assert_eq!(nested.matches(", null, false)").count(), 1, "{nested}");

        let siblings = compile(r#"<Child :a="a" /><Other :b="b" />"#, retained);
        assert_eq!(root_flags(&siblings), 0, "{siblings}");
        assert_eq!(siblings.matches(", null, false)").count(), 2, "{siblings}");

        let branches = compile(
            r#"<Child v-if="ok" :a="a" /><Other v-else :b="b" />"#,
            retained,
        );
        assert_eq!(root_flags(&branches), 2, "{branches}");

        let nested_branch = compile(
            r#"<section><p /><Child v-if="ok" :a="a" /></section>"#,
            retained,
        );
        assert_eq!(root_flags(&nested_branch), 0, "{nested_branch}");
    }
}

#[test]
fn a_keyed_or_wrapped_single_root_component_stays_the_root() {
    for retained in [false, true] {
        let keyed = compile(r#"<Child :key="k" :a="a" />"#, retained);
        assert_eq!(root_flags(&keyed), 1, "{keyed}");

        let keyed_branch = compile(r#"<Child v-if="ok" :key="k" :a="a" />"#, retained);
        assert_eq!(root_flags(&keyed_branch), 1, "{keyed_branch}");

        let template_branch = compile(
            r#"<template v-if="ok"><Child :a="a" /></template>"#,
            retained,
        );
        assert_eq!(root_flags(&template_branch), 1, "{template_branch}");

        let transition = compile(r#"<Transition><Child :a="a" /></Transition>"#, retained);
        assert_eq!(root_flags(&transition), 1, "{transition}");
    }
}
