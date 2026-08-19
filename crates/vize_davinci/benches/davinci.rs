//! Davinci microbenches: the pass manager's zero-cost-when-unattached claim.
//!
//! Run with: cargo bench -p vize_davinci --bench davinci
//!
//! The pair exists to make P2-3's central claim measurable rather than
//! asserted. Observers are dispatched statically, so `NoObserver`'s empty
//! bodies should inline away entirely and running a plan *through the
//! observer driver* should cost exactly what running the same plan without it
//! costs:
//!
//! - `davinci_pipeline_unobserved` — walks the plan directly, no driver, no
//!   observer type in sight. This is the floor.
//! - `davinci_pipeline_no_observer` — the same plan through
//!   [`run_pipeline`] with [`NoObserver`].
//!
//! **The gate is the `allocs` field**, which `budgets.toml` enforces exactly
//! and which is machine-independent: the two entries must carry the *same*
//! measured count, and `tools/davinci/bench-compare.mjs` fails any drift in
//! either. Wall times are report-only until the Blacksmith recording (P0-4),
//! and would in any case be the wrong instrument for "did this inline away" on
//! a laptop.
//!
//! The fixture pipeline is deliberately shaped like a real one — two fused
//! optional passes, a mandatory barrier, one more optional pass, so the driver
//! crosses three walks — rather than a single pass, which would measure the
//! call and not the loop.

use criterion::{Criterion, criterion_group};
use vize_davinci::pass::{
    Fusability, NoObserver, PassDesc, PassKind, Pipeline, Preserved, run_pipeline,
};

const NORMALIZE: PassDesc = PassDesc::new(
    "normalize",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const FOLD: PassDesc = PassDesc::new(
    "fold",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const VERIFY: PassDesc = PassDesc::new(
    "verify",
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    Preserved::ALL,
);
const HOIST: PassDesc = PassDesc::new(
    "hoist",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);

const PASSES: &[PassDesc] = &[NORMALIZE, FOLD, VERIFY, HOIST];
const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

/// Three walks: `normalize`+`fold`, `verify`, `hoist`.
const _: () = assert!(PIPELINE.group_count() == 3);

/// The pass body both cases run, so the two benches differ only in the driver.
///
/// Summing the name lengths keeps the body from being optimized out while
/// allocating nothing, which is what lets the alloc counts be compared at all.
#[inline]
fn work(desc: PassDesc) -> usize {
    desc.name.len()
}

fn davinci(criterion: &mut Criterion) {
    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_pipeline_unobserved",
        "crates/vize_davinci/benches/davinci.rs",
        || {
            let mut total = 0usize;
            let group_count = PIPELINE.group_count();
            for group_index in 0..group_count {
                let group = PIPELINE
                    .group(group_index)
                    .expect("group index is in range");
                for pass_index in group.start..group.end() {
                    total += work(PIPELINE.passes[pass_index]);
                }
            }
            total
        },
    );

    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_pipeline_no_observer",
        "crates/vize_davinci/benches/davinci.rs",
        || {
            let mut total = 0usize;
            run_pipeline(&PIPELINE, &mut NoObserver, |event| {
                total += work(event.desc());
                Ok(())
            })
            .expect("no step fails");
            total
        },
    );
}

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
