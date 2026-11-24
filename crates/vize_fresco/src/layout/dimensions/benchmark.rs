//! Same-process A/B of the frozen std adapter and the actual candidate adapters.
//! Construction and layout are measured separately from parsing alone.

use std::{hint::black_box, time::Instant};

use serde_json::json;

use super::super::{Dimension, FlexStyle, LayoutEngine, LengthPercentageAuto};
use super::tests::{dimension_bits, length_bits};
use super::{parse_dimension, parse_length_percentage_auto, parse_positive_point_width, reference};

trait Parser {
    fn dimension(value: &str) -> Dimension;
    fn length(value: &str) -> LengthPercentageAuto;
    fn input_width(value: &str) -> Option<usize>;
}

struct Standard;
struct Candidate;

impl Parser for Standard {
    fn dimension(value: &str) -> Dimension {
        reference::dimension(value)
    }

    fn length(value: &str) -> LengthPercentageAuto {
        reference::length(value)
    }

    fn input_width(value: &str) -> Option<usize> {
        reference::positive_width(value)
    }
}

impl Parser for Candidate {
    fn dimension(value: &str) -> Dimension {
        parse_dimension(value)
    }

    fn length(value: &str) -> LengthPercentageAuto {
        parse_length_percentage_auto(value)
    }

    fn input_width(value: &str) -> Option<usize> {
        parse_positive_point_width(value)
    }
}

#[derive(Clone, Copy, serde::Serialize)]
struct AuthoredStyle {
    width: &'static str,
    height: &'static str,
    basis: &'static str,
    min_width: &'static str,
    max_width: &'static str,
    min_height: &'static str,
    max_height: &'static str,
    inset: [&'static str; 4],
}

fn workload(count: usize, decimal: bool) -> Vec<AuthoredStyle> {
    let widths = if decimal {
        ["12.5", "33.333333%", "80.125", "50.25%"]
    } else {
        ["12", "33%", "80", "50%"]
    };
    let heights = if decimal {
        ["1.5", "2.25", "3.125", "4.75"]
    } else {
        ["1", "2", "3", "4"]
    };
    (0..count)
        .map(|index| AuthoredStyle {
            width: widths[index % widths.len()],
            height: heights[index % heights.len()],
            basis: "auto",
            min_width: if decimal { "0.125" } else { "1" },
            max_width: "100%",
            min_height: "1",
            max_height: "auto",
            inset: if decimal {
                ["0.25", "0.5%", "auto", "1.125"]
            } else {
                ["0", "1%", "auto", "1"]
            },
        })
        .collect()
}

fn style<P: Parser>(value: &AuthoredStyle) -> FlexStyle {
    FlexStyle {
        width: P::dimension(value.width),
        height: P::dimension(value.height),
        flex_basis: P::dimension(value.basis),
        min_width: P::dimension(value.min_width),
        max_width: P::dimension(value.max_width),
        min_height: P::dimension(value.min_height),
        max_height: P::dimension(value.max_height),
        inset: super::super::Inset {
            top: P::length(value.inset[0]),
            right: P::length(value.inset[1]),
            bottom: P::length(value.inset[2]),
            left: P::length(value.inset[3]),
        },
        ..FlexStyle::default()
    }
}

fn digest_style(style: &FlexStyle) -> u64 {
    let mut digest = 0_u64;
    for dimension in [
        style.width,
        style.height,
        style.flex_basis,
        style.min_width,
        style.max_width,
        style.min_height,
        style.max_height,
    ] {
        let (tag, bits) = dimension_bits(dimension);
        digest = digest
            .wrapping_mul(16777619)
            .wrapping_add(u64::from(bits) + u64::from(tag));
    }
    for length in [
        style.inset.top,
        style.inset.right,
        style.inset.bottom,
        style.inset.left,
    ] {
        let (tag, bits) = length_bits(length);
        digest = digest
            .wrapping_mul(16777619)
            .wrapping_add(u64::from(bits) + u64::from(tag));
    }
    digest
}

fn parse_nodes<P: Parser>(nodes: &[AuthoredStyle]) -> u64 {
    let mut digest = 0_u64;
    for (index, node) in nodes.iter().enumerate() {
        digest = digest.wrapping_add(digest_style(&black_box(style::<P>(black_box(node)))));
        // Input's intrinsic-width adapter is exercised on one in sixteen nodes.
        if index % 16 == 0 {
            digest =
                digest.wrapping_add(P::input_width(black_box(node.width)).unwrap_or(30) as u64);
        }
    }
    black_box(digest)
}

fn layout_nodes<P: Parser>(nodes: &[AuthoredStyle]) -> u64 {
    let mut engine = LayoutEngine::new();
    let root = engine.new_node(&FlexStyle {
        width: Dimension::Points(120.0),
        height: Dimension::Points(60_000.0),
        ..FlexStyle::default()
    });
    engine.set_root(root);
    for node in nodes {
        let id = engine.new_node(&black_box(style::<P>(black_box(node))));
        engine.add_child(root, id);
    }
    engine.compute(120.0, 60_000.0);
    assert_eq!(engine.layouts().len(), nodes.len() + 1);
    let mut digest = 0_u64;
    for id in 0..=nodes.len() as u64 {
        let rect = engine.layout(id).unwrap();
        for coordinate in [rect.x, rect.y, rect.width, rect.height] {
            digest = digest
                .wrapping_mul(16777619)
                .wrapping_add(u64::from(coordinate));
        }
    }
    black_box(digest)
}

fn measure(call: impl Fn() -> u64, iterations: usize) -> (u128, u64) {
    let start = Instant::now();
    let mut digest = 0_u64;
    for _ in 0..iterations {
        digest = digest.wrapping_add(black_box(call()));
    }
    (start.elapsed().as_nanos(), digest)
}

#[test]
#[ignore = "controlled benchmark; writes raw timings only when explicitly requested"]
fn paired_parse_and_layout() {
    let output = std::env::var("FRESCO_DIMENSION_BENCH_OUT").expect("benchmark output path");
    let mut rows = Vec::new();
    for count in [1000, 10_000] {
        for decimal in [false, true] {
            let nodes = workload(count, decimal);
            assert_eq!(
                parse_nodes::<Standard>(&nodes),
                parse_nodes::<Candidate>(&nodes)
            );
            assert_eq!(
                layout_nodes::<Standard>(&nodes),
                layout_nodes::<Candidate>(&nodes)
            );
            for layout in [false, true] {
                let iterations = if layout {
                    300_000 / count
                } else {
                    1_000_000 / count
                };
                let baseline = || {
                    if layout {
                        layout_nodes::<Standard>(&nodes)
                    } else {
                        parse_nodes::<Standard>(&nodes)
                    }
                };
                let candidate = || {
                    if layout {
                        layout_nodes::<Candidate>(&nodes)
                    } else {
                        parse_nodes::<Candidate>(&nodes)
                    }
                };
                for _ in 0..2 {
                    assert_eq!(
                        measure(baseline, iterations).1,
                        measure(candidate, iterations).1
                    );
                }
                let mut pairs = Vec::new();
                for round in 0..9 {
                    let (base, head) = if round % 2 == 0 {
                        (
                            measure(baseline, iterations),
                            measure(candidate, iterations),
                        )
                    } else {
                        let head = measure(candidate, iterations);
                        (measure(baseline, iterations), head)
                    };
                    assert_eq!(base.1, head.1);
                    pairs.push(json!({
                        "round": round, "candidate_first": round % 2 != 0,
                        "baseline_ns": base.0, "candidate_ns": head.0, "digest": head.1,
                    }));
                }
                rows.push(json!({
                    "nodes": count, "profile": if decimal { "decimal" } else { "integer" },
                    "stage": if layout { "parse_build_compute_layout" } else { "parse_only" },
                    "iterations_per_sample": iterations,
                    "authored_pattern": nodes.iter().take(4).collect::<Vec<_>>(),
                    "pairs": pairs,
                }));
            }
        }
    }
    let result = json!({
        "schema": 1,
        "base_sha": std::env::var("BASE_SHA").expect("exact base SHA"),
        "head_sha": std::env::var("HEAD_SHA").expect("exact head SHA"),
        "replica": std::env::var("REPLICA").unwrap_or_default(),
        "architecture": std::env::consts::ARCH,
        "baseline": "frozen std f32 adapters; same process, same LayoutEngine",
        "candidate": "production dimension adapters with fast-float2 0.2.4",
        "layout_scope": "parse styles, build real Taffy-backed Fresco tree, compute and read every rectangle; no Node bridge or terminal IO",
        "dimension_fields_per_node": 11,
        "input_intrinsic_nodes": "every sixteenth node",
        "warmup_pairs": 2, "measurement_pairs": 9,
        "rows": rows,
    });
    std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
