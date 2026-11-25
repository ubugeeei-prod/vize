//! Davinci fact microbenches: the fact verify mode's zero-cost-when-off
//! claim (P4-1b, the P2-3 pair shape).
//!
//! Run with: cargo bench -p vize_l0 --bench fact_runtime
//!
//! Both cases run the same plan over the same artifact — compute a demand,
//! run three passes with post-hoc `Preserved` invalidation, recompute, query
//! — and differ only in the verify policy type handed to
//! `AnalysisManager::after_pass`:
//!
//! - `davinci_fact_query_unobserved` — [`NoAnalysisVerify`], trust every claim.
//! - `davinci_fact_query_observed` — [`AnalysisVerifyObserver`], which is the
//!   recompute-and-compare mode in debug builds and compiles to nothing in
//!   the release profile benches build with.
//!
//! **The gate is the `allocs` field**: `budgets.toml` pins both entries to
//! the same measured count, so a verify observer that starts allocating in
//! release fails the exact alloc gate.

#![expect(
    clippy::expect_used,
    reason = "benchmark setup aborts on a broken fixture"
)]

use criterion::{Criterion, criterion_group};
use vize_l0::analysis::{
    Analysis, AnalysisConsumer, AnalysisManager, AnalysisProducer, AnalysisRegistry, AnalysisTable,
    AnalysisVerify, AnalysisVerifyObserver, AnalysisView, Demand, NoAnalysisVerify, ProducerEntry,
};
use vize_l0::pass::{AnalysisId, Fusability, PassDesc, PassKind, Preserved};

struct Values;
impl Analysis for Values {
    const ID: AnalysisId = AnalysisId::new(32);
    const NAME: &'static str = "values";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = u32;
}
impl AnalysisProducer<[u32]> for Values {
    fn produce(numbers: &[u32], _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        (0u32..).zip(numbers.iter().copied()).collect()
    }
}

struct Total;
impl Analysis for Total {
    const ID: AnalysisId = AnalysisId::new(33);
    const NAME: &'static str = "total";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Values::ID);
    type Key = ();
    type Value = u64;
}
impl AnalysisProducer<[u32]> for Total {
    fn produce(_: &[u32], inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let values = inputs.get::<Values>().expect("declared input");
        let total = values.iter().map(|(_, value)| u64::from(*value)).sum();
        [((), total)].into_iter().collect()
    }
}

const REGISTRY: AnalysisRegistry<[u32]> =
    AnalysisRegistry::new(&[ProducerEntry::of::<Values>(), ProducerEntry::of::<Total>()]);

struct TotalRule;
impl AnalysisConsumer for TotalRule {
    const NAME: &'static str = "total-rule";
    const DEMAND: Demand = Demand::NONE.with(Total::ID);
}

/// Keeps everything, keeps the values, keeps nothing — one of each shape.
const PASSES: [PassDesc; 3] = [
    PassDesc::new(
        "normalize",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    ),
    PassDesc::new(
        "fold",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::NONE.with(Values::ID),
    ),
    PassDesc::new(
        "rewrite",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::NONE,
    ),
];

const NUMBERS: [u32; 16] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3];

/// The plan both cases run; only `V` differs.
fn run<V: AnalysisVerify>() -> u64 {
    let mut manager = AnalysisManager::new(&REGISTRY);
    let mut total = 0u64;
    manager
        .compute(&NUMBERS[..], TotalRule::DEMAND)
        .expect("registered demand");
    for pass in &PASSES {
        manager
            .after_pass::<V>(&NUMBERS[..], pass)
            .expect("honest passes");
        let view = manager
            .prepare::<TotalRule>(&NUMBERS[..])
            .expect("registered demand");
        total += view
            .get::<Total>()
            .expect("declared")
            .get(&())
            .copied()
            .unwrap_or(0);
    }
    total
}

fn fact_runtime(criterion: &mut Criterion) {
    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_fact_query_unobserved",
        "davinci/vize_l0/benches/fact_runtime.rs",
        run::<NoAnalysisVerify>,
    );
    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_fact_query_observed",
        "davinci/vize_l0/benches/fact_runtime.rs",
        run::<AnalysisVerifyObserver>,
    );
}

criterion_group!(fact_runtime_group, fact_runtime);
davinci_harness::main!(fact_runtime_group);
