//! Directive tables the host accepts: disjoint tuples, unmapped gaps ignored.

use super::super::protocol::{DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE};
use super::transform;

/// The host rejects a file whose directive tuples overlap. Sibling expects
/// interleave their checks, and an echo ignore may sit inside an expect span.
fn overlapping_tuples(source: &str) -> Vec<(usize, usize)> {
    let result = transform(source);
    let mut intervals = result
        .diagnostic_directives
        .expect("directives")
        .directives
        .iter()
        .map(|row| (row.0[2], row.0[3]))
        .filter(|(start, end)| start < end)
        .collect::<Vec<_>>();
    intervals.sort_unstable();
    intervals
        .windows(2)
        .filter_map(|pair| match pair {
            [left, right] if right.0 < left.1 => Some(*right),
            _ => None,
        })
        .collect()
}

#[test]
fn sibling_component_expects_keep_disjoint_intervals() {
    let source = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\n</script>\n<template>\n  <!-- @vue-expect-error -->\n  <Child :n=\"'a'\" />\n  <!-- @vue-expect-error -->\n  <Child :n=\"'b'\" />\n</template>\n";
    assert_eq!(overlapping_tuples(source), Vec::<(usize, usize)>::new());
}

#[test]
fn expect_on_an_unknown_component_keeps_echo_ignores_disjoint() {
    let source = "<script setup lang=\"ts\">\nimport { ref } from 'vue'\nconst msg = ref('a')\n</script>\n<template>\n  <!-- @vue-expect-error -->\n  <Hello :msg=\"msg\"></Hello>\n</template>\n";
    assert_eq!(overlapping_tuples(source), Vec::<(usize, usize)>::new());
}

#[test]
fn unmapped_ignores_fill_only_uncovered_virtual_gaps() {
    use super::super::protocol::{
        ContentMapperDiagnosticDirective as Row, ContentMapperDiagnosticDirectives as Table,
        ContentMapperSpan as Span,
    };
    let spans = [
        Span([5, 5, 0, 5, 0, 0]),
        Span([20, 10, 9, 10, 0, 0]),
        Span([12, 0, 3, 0, 0, 0]),
    ];
    let expect = Row([1, 2, 22, 35, DIRECTIVE_POLICY_EXPECT, 0]);
    let table = Table {
        directives: vec![expect],
        unused_expect_directive_diagnostics: Vec::new(),
    };
    let result =
        super::super::unmapped_ignores::unmapped_ignores(Some(table), &spans, 40).expect("table");
    assert_eq!(
        result.directives,
        [
            Row([0, 0, 0, 5, DIRECTIVE_POLICY_IGNORE, 0]),
            Row([0, 0, 10, 20, DIRECTIVE_POLICY_IGNORE, 0]),
            expect,
            Row([0, 0, 35, 40, DIRECTIVE_POLICY_IGNORE, 0]),
        ]
    );
    assert_eq!(
        super::super::unmapped_ignores::unmapped_ignores(None, &[Span([0, 40, 0, 40, 0, 0])], 40),
        None
    );
}
