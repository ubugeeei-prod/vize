//! Davinci fact microbenches: the fact verify mode's zero-cost-when-off
//! claim (P4-1b, the P2-3 pair shape).
//!
//! Run with: cargo bench -p vize_davinci --bench davinci_fact
//!
//! Both cases run the same plan over the same artifact — compute a demand,
//! run three passes with post-hoc `Preserved` invalidation, recompute, query
//! — and differ only in the verify policy type handed to
//! `FactManager::after_pass`:
//!
//! - `davinci_fact_query_unobserved` — [`NoFactVerify`], trust every claim.
//! - `davinci_fact_query_observed` — [`FactVerifyObserver`], which is the
//!   recompute-and-compare mode in debug builds and compiles to nothing in
//!   the release profile benches build with.
//!
//! **The gate is the `allocs` field**: `budgets.toml` pins both entries to
//! the same measured count, so a verify observer that starts allocating in
//! release fails the exact alloc gate.

use criterion::{Criterion, criterion_group};
use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactManager, FactProducer, FactRegistry, FactTable,
    FactVerify, FactVerifyObserver, FactView, NoFactVerify, ProducerEntry,
};
use vize_davinci::pass::{AnalysisId, Fusability, PassDesc, PassKind, Preserved};

struct Values;
impl FactGroup for Values {
    const ID: AnalysisId = AnalysisId::new(32);
    const NAME: &'static str = "values";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = u32;
}
impl FactProducer<[u32]> for Values {
    fn produce(numbers: &[u32], _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(numbers.iter().copied()).collect()
    }
}

struct Total;
impl FactGroup for Total {
    const ID: AnalysisId = AnalysisId::new(33);
    const NAME: &'static str = "total";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Values::ID);
    type Key = ();
    type Value = u64;
}
impl FactProducer<[u32]> for Total {
    fn produce(_: &[u32], inputs: &FactView<'_>) -> FactTable<Self> {
        let values = inputs.get::<Values>().expect("declared input");
        let total = values.iter().map(|(_, value)| u64::from(*value)).sum();
        [((), total)].into_iter().collect()
    }
}

const REGISTRY: FactRegistry<[u32]> =
    FactRegistry::new(&[ProducerEntry::of::<Values>(), ProducerEntry::of::<Total>()]);

struct TotalRule;
impl FactConsumer for TotalRule {
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
fn run<V: FactVerify>() -> u64 {
    let mut manager = FactManager::new(&REGISTRY);
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

fn davinci_fact(criterion: &mut Criterion) {
    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_fact_query_unobserved",
        "crates/vize_davinci/benches/davinci_fact.rs",
        run::<NoFactVerify>,
    );
    davinci_harness::bench_with_metrics(
        criterion,
        "davinci_fact_query_observed",
        "crates/vize_davinci/benches/davinci_fact.rs",
        run::<FactVerifyObserver>,
    );
}

criterion_group!(davinci_fact_group, davinci_fact);
davinci_harness::main!(davinci_fact_group);
