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
    let phase = args
        .next()
        .expect("phase: controls, inspect, guard, guarded, or raw");
    if phase == "controls" {
        let path = args.next().expect("exact input path");
        assert!(args.next().is_none());
        control_packets(&fs::read_to_string(path).expect("UTF-8 reproducer"));
        return;
    }
    let path = args.next().expect("input path");
    let iterations: usize = args
        .next()
        .map_or(1, |value| value.parse().expect("integer iteration count"));
    assert!((1..=100_000).contains(&iterations));
    assert!(args.next().is_none());
    let source = fs::read_to_string(path).expect("UTF-8 reproducer");
    if phase == "inspect" {
        println!(
            "{}",
            serde_json::json!({
                "bytes": source.len(),
                "safe": expression_is_safe_to_parse(&source),
                "depth": expression_nesting_depth(&source),
                "balanced": expression_has_balanced_delimiters(&source),
            })
        );
        return;
    }
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

fn control_packets(reproducer: &str) {
    let controls = [
        "f<T>(arg)",
        "f<Map<string, Array<number>>>(value)",
        "f<A<B<C>>>(value)",
        "a > b ? f<T>(x) : g<U>(y)",
        "f<T /* > */>(value)",
        "f<\"greater >\">(value)",
        "f<`greater >`>(value)",
        "f<`head ${T} >`>(value)",
        "f<T>(\">\")",
        "f<T>`greater >`",
        "a < b",
        "a < b && c < d",
        "a < b > c",
        "<T",
        "<T>value",
        "f<T",
        "f<[[A, B]",
        "f<T,>() < U",
        "f</* > */ T",
        "f<'>',>() < U",
        "f<'\\u003e'>(x)",
        "f<\\u0054>(x)",
        "f\\u003cT",
        "<Widget value={left > right} />",
        "<Widget<T> value={x} />",
        "<T,>(arg: T) => arg",
        "cond ? <Widget /> : <Other />",
        "<Box title=\"a > b\" />",
        "f<T>(x) > g<U>(y)",
        "f<T>() < U",
    ];
    let mut packets = Vec::new();
    for (profile, source_type) in [
        ("ts", SourceType::ts()),
        ("tsx", SourceType::tsx()),
        ("jsx", SourceType::jsx()),
    ] {
        for source in controls {
            let allocator = Allocator::default();
            let result = Parser::new(&allocator, source, source_type).parse_expression();
            // Debug includes the complete AST with spans or diagnostic packets
            // with labels. Compare exact packets across the two source builds.
            packets.push(serde_json::json!({
                "profile": profile,
                "source": source,
                "packet": format!("{result:?}"),
            }));
        }
    }
    let allocator = Allocator::default();
    let result = Parser::new(&allocator, reproducer, SourceType::ts()).parse_expression();
    packets.push(serde_json::json!({
        "profile": "ts",
        "source": "exact issue-10401 artifact",
        "packet": format!("{result:?}"),
    }));
    println!("{}", serde_json::to_string(&packets).unwrap());
}
