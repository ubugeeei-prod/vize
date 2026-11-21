use super::*;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired timings over an original checked Program projection"]
fn paired_queries() {
    for (name, atom) in [("ascii", "just source   "), ("unicode", "雪文😀 author ")] {
        for rows in [1, 64, 512] {
            let text = vize_l0::cstr!("/* {} */ const value = 1;", atom.repeat(rows));
            let arena = Allocator::default();
            let file = program::file(&arena, &text, Lang::Js).unwrap();
            let projection = project_program(&file).unwrap();
            let size = utf16_len(&text) as u32;
            for count in [1, 16, 128] {
                let forward: Vec<_> = (0..count)
                    .map(|n| ((n + 1) * (size - 1) / (count + 1), 1))
                    .collect();
                let mut backward = forward.clone();
                backward.reverse();
                let mixed: Vec<_> = (0..count as usize)
                    .map(|i| forward[(i * 53) % forward.len()])
                    .collect();
                for (order, points) in [
                    ("forward", &forward),
                    ("backward", &backward),
                    ("unordered", &mixed),
                ] {
                    // Amortize each clock read for short, inexpensive queries.
                    let iterations = if count == 1 { 512 } else { 16 };
                    let time = |batch| {
                        let start = Instant::now();
                        for _ in 0..iterations {
                            if batch {
                                let mut mapper = black_box(&projection).diagnostic_mapper();
                                for &(a, b) in black_box(points) {
                                    let _ = black_box(mapper.map_utf16(a, b));
                                }
                            } else {
                                for &(a, b) in black_box(points) {
                                    let _ = black_box(projection.map_utf16(a, b));
                                }
                            }
                        }
                        start.elapsed().as_nanos() / iterations
                    };
                    let mut pairs = Vec::new();
                    for i in 0..35 {
                        let reverse = i % 2 != 0;
                        let a = time(reverse);
                        let b = time(!reverse);
                        if i >= 4 {
                            pairs.push(if reverse { (b, a) } else { (a, b) });
                        }
                    }
                    eprintln!(
                        "family={name} rows={rows} requests={count} order={order} pairs={pairs:?}"
                    );
                }
            }
        }
    }
}

pub(super) fn original_batch<'p, M: Fn(Span) -> Result<Span, MappingError>>(
    doc: &EmitDocument,
    points: &[u32],
    make: &impl Fn() -> DiagnosticMapper<'p, M>,
    original: &impl Fn(u32, u32) -> Result<Span, MappingError>,
    source: usize,
) {
    // Diagnostics commonly arrive in source order. Keep at most 12 anchors
    // per document and include range checks in both arms.
    let queries: Vec<_> = points
        .iter()
        .copied()
        .filter(|p| *p != u32::MAX)
        .step_by(points.len().div_ceil(12).max(1))
        .take(12)
        .collect();
    let time = |group| {
        let start = Instant::now();
        for _ in 0..4 {
            if group {
                let mut mapper = black_box(make)();
                for &at in black_box(&queries) {
                    let _ = black_box(mapper.map_utf16(at, 1));
                }
            } else {
                for &at in black_box(&queries) {
                    let _ = black_box(original(at, 1));
                }
            }
        }
        start.elapsed().as_nanos() / 4
    };
    let mut pairs = Vec::new();
    for i in 0..13 {
        let reverse = i % 2 != 0;
        let a = time(reverse);
        let b = time(!reverse);
        if i >= 4 {
            pairs.push(if reverse { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "doc={source} links={} bytes={} count={} pairs={pairs:?}",
        doc.is_recording(),
        doc.len(),
        queries.len()
    );
}
