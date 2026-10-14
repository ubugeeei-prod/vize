//! `createIf` and `createFor` carry the block shapes the Vapor runtime reads
//! while hydrating, as Vue's compiler encodes them.

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
fn template_loop_rows_are_wrapped_fragments() {
    for source in [
        r#"<template v-for="i in l"><span>{{ i }}</span><b></b></template>"#,
        r#"<template v-for="i in l" :key="i"><span>a</span><b></b></template>"#,
        r#"<div><template v-for="i in l" :key="i">{{ i }}</template></div>"#,
    ] {
        for retained in [false, true] {
            let code = compile(source, retained);
            assert!(
                code.contains("WRAPPED_ROWS */)"),
                "{retained} {source}\n{code}"
            );
        }
    }
}

#[test]
fn element_loops_and_single_element_template_rows_are_not_wrapped() {
    for source in [
        r#"<li v-for="i in l" :key="i">{{ i }}</li>"#,
        r#"<template v-for="i in l" :key="i"><li>{{ i }}</li></template>"#,
    ] {
        for retained in [false, true] {
            let code = compile(source, retained);
            assert!(
                !code.contains("WRAPPED_ROWS"),
                "{retained} {source}\n{code}"
            );
            assert!(
                code.contains("8 /* IS_SINGLE_NODE */)"),
                "{retained} {source}\n{code}"
            );
        }
    }
}

#[test]
fn branch_shapes_follow_their_roots() {
    let cases = [
        (
            r#"<template v-if="a"><b></b><i></i></template><p v-else>c</p>"#,
            "6 /* TRUE_MULTI_ROOT, FALSE_SINGLE_ROOT */)",
        ),
        (
            r#"<template v-if="a">text</template>"#,
            "2 /* TRUE_MULTI_ROOT */)",
        ),
        (
            r#"<p v-if="a">a</p><p v-else-if="b">b</p>"#,
            "5 /* TRUE_SINGLE_ROOT, FALSE_SINGLE_ROOT */)",
        ),
    ];
    for (source, flags) in cases {
        for retained in [false, true] {
            let code = compile(source, retained);
            assert!(code.contains(flags), "{retained} {source}\n{code}");
        }
    }
    for retained in [false, true] {
        let code = compile(r#"<p v-if="a">a</p>"#, retained);
        assert!(code.contains("})"), "{code}");
        assert!(
            !code.contains("/* TRUE_"),
            "the runtime default needs no flags\n{code}"
        );
    }
}

#[test]
fn a_conditional_loop_row_is_a_fragment_in_both_branches() {
    let source = r#"<template v-for="i in l"><div v-if="i.ok">{{ i }}</div></template>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            code.contains("10 /* TRUE_MULTI_ROOT, FALSE_MULTI_ROOT */)"),
            "{code}"
        );
        assert!(
            code.contains("80 /* IS_FRAGMENT, WRAPPED_ROWS */)"),
            "{code}"
        );
    }
}

#[test]
fn a_loop_that_is_its_parents_only_child_removes_fast() {
    for retained in [false, true] {
        let code = compile(
            r#"<ul><li v-for="i in l" :key="i">{{ i }}</li></ul>"#,
            retained,
        );
        assert!(
            code.contains("9 /* FAST_REMOVE, IS_SINGLE_NODE */)"),
            "{code}"
        );
        let code = compile(
            r#"<ul><li v-for="i in l" :key="i">{{ i }}</li><li>last</li></ul>"#,
            retained,
        );
        assert!(!code.contains("FAST_REMOVE"), "{code}");
    }
}
