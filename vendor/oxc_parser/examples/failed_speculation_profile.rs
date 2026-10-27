//! Frozen driver for immutable-source failure-sharing audits.
use std::{env, fs, hint::black_box, time::Instant};

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser, config::RuntimeParserConfig};
use oxc_span::SourceType;

fn main() {
    let mut args = env::args().skip(1);
    let mode = args.next().expect("mode: time or packets");
    let path = args.next().expect("UTF-8 input path");
    let source = fs::read_to_string(path).unwrap();
    if mode == "packets" {
        assert!(args.next().is_none());
        packets(&source);
        return;
    }
    assert_eq!(mode, "time");
    let iterations: usize = args.next().map_or(1, |n| n.parse().unwrap());
    assert!((1..=100_000).contains(&iterations));
    assert!(args.next().is_none());
    let start = Instant::now();
    let mut rejected = 0;
    let mut peak_arena_used_bytes = 0;
    for _ in 0..iterations {
        let allocator = Allocator::default();
        rejected += usize::from(
            black_box(
                Parser::new(&allocator, black_box(&source), SourceType::ts()).parse_expression(),
            )
            .is_err(),
        );
        peak_arena_used_bytes = peak_arena_used_bytes.max(allocator.used_bytes());
    }
    println!(
        "{{\"iterations\":{iterations},\"rejected\":{rejected},\"elapsed_ns\":{},\"peak_arena_used_bytes\":{peak_arena_used_bytes}}}",
        start.elapsed().as_nanos()
    );
}

fn packets(source: &str) {
    for source_type in [
        SourceType::ts(),
        SourceType::tsx(),
        SourceType::jsx(),
        SourceType::ts().with_script(true),
        SourceType::ts().with_unambiguous(true),
    ] {
        for tokens in [false, true] {
            for parse_regular_expression in [false, true] {
                let options = ParseOptions { parse_regular_expression, ..ParseOptions::default() };
                let allocator = Allocator::default();
                let result = Parser::new(&allocator, source, source_type)
                    .with_config(RuntimeParserConfig::new(tokens))
                    .with_options(options)
                    .parse();
                println!(
                    "program {source_type:?} tokens={tokens} regexp={parse_regular_expression}: {:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
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
                );
                let allocator = Allocator::default();
                let result = Parser::new(&allocator, source, source_type)
                    .with_config(RuntimeParserConfig::new(tokens))
                    .with_options(options)
                    .parse_expression();
                println!(
                    "expression {source_type:?} tokens={tokens} regexp={parse_regular_expression}: {result:?}"
                );
            }
        }
    }
}
