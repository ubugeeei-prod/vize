#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use vize_atelier_core::parser::patterns::parse_match_attribute;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor_with_diagnostics};
use vize_carton::Allocator;

#[test]
fn every_backend_reports_the_shared_pattern_error_at_the_authored_span() {
    for pattern in [
        "const { value }",
        "value as const alias",
        "[const x, const x]",
        "{ value: 1, value: 2 }",
        "[1, ...rest]",
        "[1, ...const rest,]",
        "{ [key]: 1 }",
        "predicate()",
        "value + 1",
        "[const x | const y]",
        "_ if ()",
        "{ &quot;\u{e9}&quot;: const value, const value }",
    ] {
        let error = parse_match_attribute(pattern).unwrap_err();
        let source = format!(
            "<!-- \u{1f600} -->\r\n<template v-match=\"subject\"><p v-when=\"{pattern}\">bad</p><p v-when=\"_\">other</p></template>"
        );
        let start = source.find(pattern).unwrap() as u32;
        let end = start + pattern.len() as u32;
        let at = start + error.offset;
        let expected = vec![(
            error.message.as_str(),
            if at < end { at } else { start },
            end,
        )];
        let allocator = Allocator::default();
        let (_, dom, _) = compile_template_with_options(
            &allocator,
            &source,
            DomCompilerOptions {
                experimental_patterned_template: true,
                ..Default::default()
            },
        );
        let (_, ssr, _) = compile_ssr_with_options(
            &allocator,
            &source,
            SsrCompilerOptions {
                experimental_patterned_template: true,
                ..Default::default()
            },
        );
        let (vapor, diagnostics) = compile_vapor_with_diagnostics(
            &allocator,
            &source,
            VaporCompilerOptions {
                experimental_patterned_template: true,
                ..Default::default()
            },
        );
        assert_eq!(vapor.code, "", "{pattern}");
        for errors in [&dom, &ssr, &diagnostics] {
            let actual = errors
                .iter()
                .map(|error| {
                    let loc = error.loc.as_ref().unwrap();
                    (error.message.as_str(), loc.span.start, loc.span.end)
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{pattern}");
        }
    }
}

#[test]
fn canonical_arms_reject_competing_structural_directives() {
    for directive in [
        "v-if=\"ready\"",
        "v-else-if=\"ready\"",
        "v-else",
        "v-for=\"item in items\"",
        "v-match=\"nested\"",
    ] {
        let source = format!(
            "<template v-match=\"subject\"><p v-when=\"const value\" {directive}>{{{{ value }}}}</p><p v-when=\"_\">other</p></template>"
        );
        let allocator = Allocator::default();
        let (result, diagnostics) = compile_vapor_with_diagnostics(
            &allocator,
            &source,
            VaporCompilerOptions {
                experimental_patterned_template: true,
                ..Default::default()
            },
        );
        assert_eq!(result.code, "", "{directive}");
        assert_eq!(
            diagnostics[0].message,
            "v-when cannot share an element with v-if, v-else-if, v-else, v-for or v-match."
        );
        let loc = diagnostics[0].loc.as_ref().unwrap();
        assert_eq!(
            &source[loc.span.start as usize..loc.span.end as usize],
            "v-when=\"const value\""
        );
    }
}
