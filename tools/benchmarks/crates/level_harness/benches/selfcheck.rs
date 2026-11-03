//! Harness selfcheck bench (P0-1): exercises every metric family end to end.
//!
//! Run with: cargo bench -p davinci_harness
//!
//! The routine allocates one Vec, fills it, and reduces it - enough to make
//! the allocation counters, peak-bytes tracking, wall percentiles, and RSS
//! delta all take non-degenerate values in the exported report at
//! `tools/benchmarks/results/davinci/harness-selfcheck.json`.

use std::hint::black_box;

use criterion::{Criterion, criterion_group};

fn selfcheck(criterion: &mut Criterion) {
    davinci_harness::bench_with_metrics(
        criterion,
        "harness-selfcheck",
        "synthetic:vec-fill-1024",
        || {
            let mut values: Vec<u64> = Vec::with_capacity(1024);
            for i in 0..1024u64 {
                values.push(black_box(i).wrapping_mul(0x9e37_79b9));
            }
            values.iter().copied().sum::<u64>()
        },
    );
}

criterion_group!(selfcheck_group, selfcheck);
davinci_harness::main!(selfcheck_group);
