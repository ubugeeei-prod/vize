//! Bounded phase probe for the exact expression slow unit in #10401.
//!
//! Run each phase in a separate process so source profilers can distinguish
//! guard scanning from parsing. `raw` deliberately bypasses admission only in
//! this diagnostic executable; the fuzz target still uses the shared guard.

use std::{env, fs, hint::black_box, time::Instant};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::{
    expression_has_balanced_delimiters, expression_is_safe_to_parse, expression_nesting_depth,
};

fn main() {
    let mut args = env::args().skip(1);
    let phase = args.next().expect("phase: guard, guarded, or raw");
    let path = args.next().expect("input path");
    let iterations: usize = args
        .next()
        .map_or(1, |value| value.parse().expect("integer iteration count"));
    assert!((1..=100_000).contains(&iterations));
    assert!(args.next().is_none());
    let source = fs::read_to_string(path).expect("UTF-8 reproducer");
    let safe = expression_is_safe_to_parse(&source);
    eprintln!(
        "{}",
        serde_json::json!({
            "phase": phase,
            "bytes": source.len(),
            "iterations": iterations,
            "safe": safe,
            "depth": expression_nesting_depth(&source),
            "balanced": expression_has_balanced_delimiters(&source),
        })
    );
    let start = Instant::now();
    let mut accepted = 0;
    let mut rejected = 0;
    let mut skipped = 0;
    for _ in 0..iterations {
        match phase.as_str() {
            "guard" => {
                accepted += usize::from(black_box(expression_is_safe_to_parse(black_box(&source))));
            }
            "guarded" | "raw" => {
                if phase == "guarded" && !expression_is_safe_to_parse(black_box(&source)) {
                    skipped += 1;
                    continue;
                }
                let arena = Allocator::default();
                let result = Parser::new(
                    &arena,
                    black_box(&source),
                    SourceType::default()
                        .with_module(true)
                        .with_typescript(true),
                )
                .parse_expression();
                match black_box(result) {
                    Ok(_) => accepted += 1,
                    Err(_) => rejected += 1,
                }
            }
            _ => panic!("unknown phase"),
        }
    }
    println!(
        "{}",
        serde_json::json!({
            "phase": phase,
            "iterations": iterations,
            "elapsed_ns": start.elapsed().as_nanos(),
            "accepted": accepted,
            "rejected": rejected,
            "skipped": skipped,
        })
    );
}
