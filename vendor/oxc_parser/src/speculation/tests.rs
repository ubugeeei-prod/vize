use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::{
    ParseOptions, Parser, ParserConfig, ParserImpl, ParserReturn, UniquePromise,
    config::{NoTokensParserConfig, RuntimeParserConfig, TokensParserConfig},
};

fn parser<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    enabled: bool,
) -> ParserImpl<'a, TokensParserConfig> {
    let mut parser = ParserImpl::new(
        allocator,
        source,
        source_type,
        ParseOptions { parse_regular_expression: true, ..ParseOptions::default() },
        TokensParserConfig,
        UniquePromise::new_for_tests_and_benchmarks(),
    );
    parser.state.disable_failed_subproblems = !enabled;
    parser
}

fn packet(source: &str, source_type: SourceType, enabled: bool) -> String {
    let allocator = Allocator::default();
    let result = parser(&allocator, source, source_type, enabled).parse();
    complete_packet(result)
}

fn complete_packet(result: ParserReturn<'_>) -> String {
    format!(
        "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
        result.program,
        result.diagnostics,
        result.tokens,
        result.module_record,
        result.irregular_whitespaces,
        (
            result.panicked,
            result.is_flow_language,
            result.has_legacy_literals,
            result.has_jsdoc_comments
        ),
    )
}

#[test]
fn known_entry_policies_preserve_complete_packets_and_token_modes() {
    fn compare<C: ParserConfig + Copy>(source: &str, source_type: SourceType, config: C) {
        let options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
        let forced = {
            let allocator = Allocator::default();
            complete_packet(
                ParserImpl::new(
                    &allocator,
                    source,
                    source_type,
                    options,
                    config,
                    UniquePromise::new_for_tests_and_benchmarks(),
                )
                .parse(),
            )
        };
        let selected = {
            let allocator = Allocator::default();
            complete_packet(
                Parser::new(&allocator, source, source_type)
                    .with_options(options)
                    .with_config(config)
                    .parse(),
            )
        };
        assert_eq!(forced, selected, "program entry policy: {source:?} {source_type:?}");

        let forced = {
            let allocator = Allocator::default();
            format!(
                "{:?}",
                ParserImpl::new(
                    &allocator,
                    source,
                    source_type,
                    options,
                    config,
                    UniquePromise::new_for_tests_and_benchmarks(),
                )
                .parse_expression()
            )
        };
        let selected = {
            let allocator = Allocator::default();
            format!(
                "{:?}",
                Parser::new(&allocator, source, source_type)
                    .with_options(options)
                    .with_config(config)
                    .parse_expression()
            )
        };
        assert_eq!(forced, selected, "expression entry policy: {source:?} {source_type:?}");
    }

    for source in [
        "a + b",
        "({ [name]: /a>b/u })",
        "/** @type {number} */ var value = 077;",
        "// @flow\na",
        "a\u{00a0}+ b",
        "return 1",
        "type T = number | null",
        "const value = ;",
        "'literal < opener'",
        "/* < */ a",
        "f<T>(x)",
        "<T,>(x: T) => x",
    ] {
        for source_type in [
            SourceType::ts(),
            SourceType::tsx(),
            SourceType::jsx(),
            SourceType::ts().with_script(true),
            SourceType::ts().with_unambiguous(true),
        ] {
            compare(source, source_type, NoTokensParserConfig);
            compare(source, source_type, TokensParserConfig);
            compare(source, source_type, RuntimeParserConfig::new(false));
            compare(source, source_type, RuntimeParserConfig::new(true));
        }
    }
}

#[test]
fn complete_program_and_expression_packets_match_without_memoization() {
    let atoms = [
        "T",
        "[T]",
        "[[T, U], V]",
        "{ [x < y]: T }",
        "typeof import('m')",
        "{ [import.meta.url]: T }",
        "{ [import('m')]: T }",
        "{ [(() => { return /x>y/.test(x) })()]: T }",
        "{ [({ a = 1 } = obj)]: T }",
        "{ [await value]: T }",
        "[T /* > */]",
        "[T, /* @__PURE__ */ f()]",
        "[T, /* @__NO_SIDE_EFFECTS__ */ (() => 0)]",
        "`head ${T} >`",
        "'greater >'",
        "\\u0054",
        "[...T[], U?]",
        "T extends U ? V : W",
        "([x, { y }]: T) => U",
        "[",
        "[[,]<x",
        "{[[,]<x",
        "{ [a < [b < [c]]]: T }",
    ];
    let mut sources = Vec::new();
    for atom in atoms {
        for suffix in [">(x)", ">", ">= x", "]", "\n// >", ">`tail`", ">.x", "> + x"] {
            for prefix in ["f<", "a < b <", "f<[{[", "/* @__PURE__ */ f<"] {
                sources.push(format!("{prefix}{atom}{suffix}"));
            }
        }
    }
    sources.extend(
        [
            "a < [{ [(() => { import('m'); export {}; })()]: T }] > x",
            "a < [{ [(() => { const { a = 1 } = x; return a; })()]: T }] > x",
            "f<T>() < U",
            "<T,>(x: T) => x",
            "<Widget<T> value={a > b} />",
            "f<T>(x) /* @__PURE__ */ .foo()",
            "f<T>(x)\u{00a0}< U",
        ]
        .map(str::to_owned),
    );
    for source in sources {
        for source_type in [
            SourceType::ts(),
            SourceType::tsx(),
            SourceType::jsx(),
            SourceType::ts().with_script(true),
            SourceType::ts().with_unambiguous(true),
        ] {
            let baseline = packet(&source, source_type, false);
            let candidate = packet(&source, source_type, true);
            assert_eq!(baseline, candidate, "complete program packet: {source:?} {source_type:?}");
            let allocator = Allocator::default();
            let baseline =
                format!("{:?}", parser(&allocator, &source, source_type, false).parse_expression());
            let allocator = Allocator::default();
            let candidate =
                format!("{:?}", parser(&allocator, &source, source_type, true).parse_expression());
            assert_eq!(
                baseline, candidate,
                "complete expression packet: {source:?} {source_type:?}"
            );
        }
    }
}

#[test]
fn overlapping_fatal_subproblems_are_shared_without_changing_recovery() {
    let source = include_str!("fixtures/issue-10845.input");
    let mut observations = Vec::new();
    for enabled in [false, true] {
        let allocator = Allocator::default();
        let mut parser = parser(&allocator, source, SourceType::ts(), enabled);
        parser.bump_any();
        let expression = parser.parse_expr();
        assert!(parser.fatal_error.is_some());
        observations.push((
            format!("{:?}", parser.fatal_error.as_ref().unwrap().error),
            parser.state.subproblem_visits,
            parser.state.subproblem_reuses,
            allocator.used_bytes(),
        ));
        std::hint::black_box(expression);
    }
    assert_eq!(observations[0].0, observations[1].0);
    assert_eq!(observations[0].2, 0);
    assert!(observations[1].2 > 0, "no failed subproblem was reused: {observations:?}");
    assert!(
        observations[0].1 > observations[1].1 * 8,
        "overlapping work did not fall by 8x: {observations:?}"
    );
    assert!(
        observations[0].3 > observations[1].3 * 8,
        "repeated arena allocation was not eliminated: {observations:?}"
    );
    eprintln!("exact #10845 (diagnostic, visits, reuses, arena bytes): {observations:?}");
    assert_eq!(packet(source, SourceType::ts(), false), packet(source, SourceType::ts(), true));
}

#[test]
fn irreversible_side_effects_are_fenced_even_when_collection_lengths_match() {
    use super::SpeculativeProduction;
    use oxc_allocator::{Dummy, GetAllocator};
    use oxc_ast::ast::Expression;
    for writer in 0..6 {
        let allocator = Allocator::default();
        let mut parser = parser(&allocator, "[x]", SourceType::ts(), true);
        parser.bump_any();
        parser.state.type_argument_speculation_depth = 1;
        parser.state.reuse_failed_type_argument_subproblems = true;
        let _: Expression =
            parser.memoize_failed_speculative_subproblem(SpeculativeProduction::Type, |p| {
                match writer {
                    0 => p.state.speculation_effects_revision += 1,
                    1 => p.module_record_builder.speculation_effects_revision += 1,
                    2 => p.lexer.trivia_builder.speculation_effects_revision += 1,
                    3 => p.state.encountered_await_identifier = true,
                    4 => p.ctx.toggle(crate::Context::Decorator),
                    5 => {}
                    _ => unreachable!(),
                }
                p.set_unexpected();
                Dummy::dummy(p.allocator())
            });
        assert!(parser.fatal_error.is_some(), "writer {writer} did not produce a fatal error");
        // The no-effect control must populate the table, so the other cases
        // cannot pass without exercising their individual fences.
        assert_eq!(
            parser.state.failed_speculative_subproblems.len(),
            usize::from(writer == 5),
            "writer {writer} memoization mismatch"
        );
    }
}

#[test]
fn ordinary_successful_programs_do_not_allocate_failed_subproblem_tables() {
    let allocator = Allocator::default();
    let mut parser =
        parser(&allocator, "f<Map<string, Array<number>>>(value)", SourceType::ts(), true);
    parser.bump_any();
    let _ = parser.parse_expr();
    assert!(parser.fatal_error.is_none());
    assert_eq!(parser.state.failed_speculative_subproblems.capacity(), 0);
    assert_eq!(parser.state.type_argument_speculation_depth, 0);
}
