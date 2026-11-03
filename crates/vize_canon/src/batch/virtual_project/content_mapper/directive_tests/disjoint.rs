//! Directive tables the host accepts: disjoint tuples, unmapped gaps ignored.

use super::super::protocol::{DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE};
use super::{published, transform};

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

/// The CLI mapper and the napi editor projection call different entry
/// points; each must return the same table, unmapped ignores included (#11472).
#[test]
fn every_entry_point_ignores_unmapped_scaffolding() {
    use std::path::Path;

    use super::super::{
        ContentMapperTransformOptions, ContentMapperTypeSources,
        generate_vue_content_mapper_transform_with_options,
        generate_vue_content_mapper_transform_with_project_context,
        generate_vue_content_mapper_transform_with_project_options,
        generate_vue_content_mapper_transform_with_type_sources,
    };
    use crate::batch::virtual_project::ContentMapperProjectContext;

    let path = Path::new("/project/App.vue");
    let options = ContentMapperTransformOptions::default();
    let context = ContentMapperProjectContext::default();
    for (source, fallback) in [
        (
            "<script setup lang=\"ts\">\nconst n: number = 1\n</script>\n<template>\n  <p>{{ n }}</p>\n</template>\n",
            false,
        ),
        // The invalid-SFC fallback has no mappings at all.
        ("<script setup lang=\"ts\">\nconst n = 1\n", true),
    ] {
        let complete = published(source);
        assert_eq!(complete.mappings.is_empty(), fallback, "{source}");
        assert_complete_table(&[
            complete,
            generate_vue_content_mapper_transform_with_options(path, source, options).unwrap(),
            generate_vue_content_mapper_transform_with_project_options(path, source, options, None)
                .unwrap(),
            generate_vue_content_mapper_transform_with_project_context(
                path, source, options, &context,
            )
            .unwrap(),
            generate_vue_content_mapper_transform_with_type_sources(
                path,
                source,
                options,
                &context,
                &ContentMapperTypeSources::default(),
            )
            .unwrap(),
        ]);
    }
}

fn assert_complete_table(transforms: &[super::super::ContentMapperTransform]) {
    let first = &transforms[0];
    let table = first
        .diagnostic_directives
        .as_ref()
        .expect("directive table");
    let mut mapped = vec![false; first.text.len()];
    for span in &first.mappings {
        let [start, length, ..] = span.0;
        mapped[start..start + length].fill(true);
    }
    let mut covered = mapped.clone();
    for row in &table.directives {
        let [_, _, start, end, policy, _] = row.0;
        assert!(policy == DIRECTIVE_POLICY_IGNORE || policy == DIRECTIVE_POLICY_EXPECT);
        covered[start..end].fill(true);
    }
    assert!(
        mapped.iter().any(|byte| !byte),
        "the projection has scaffolding"
    );
    assert!(
        covered.iter().all(|byte| *byte),
        "every unmapped byte is ignored"
    );
    for other in &transforms[1..] {
        assert_eq!(other.text, first.text);
        assert_eq!(other.diagnostic_directives, first.diagnostic_directives);
    }
}
