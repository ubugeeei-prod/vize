//! Fixture fact groups over a `[u32]` artifact.
//!
//! ```text
//! stratum 0   values            unused (registered, never demanded)
//! stratum 1   squares(values)   parity(values)
//! stratum 2   summary(squares, parity)
//! ```

use std::sync::{Mutex, MutexGuard};

use vize_l0::analysis::{
    Analysis, AnalysisConsumer, AnalysisProducer, AnalysisRegistry, AnalysisTable, AnalysisView,
    Demand, ProducerEntry,
};
use vize_l0::pass::AnalysisId;

/// Serializes the tests that read the process-global counters or the
/// production log.
static LOCK: Mutex<()> = Mutex::new(());
/// Producer names in run order.
static LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

pub fn serial() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn take_log() -> Vec<&'static str> {
    std::mem::take(
        &mut *LOG
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
}

fn log(name: &'static str) {
    LOG.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(name);
}

macro_rules! group {
    ($ty:ident, $id:expr, $name:literal, $stratum:expr, $depends:expr, $value:ty) => {
        pub struct $ty;
        impl Analysis for $ty {
            const ID: AnalysisId = AnalysisId::new($id);
            const NAME: &'static str = $name;
            const STRATUM: u8 = $stratum;
            const DEPENDS: Demand = $depends;
            type Key = u32;
            type Value = $value;
        }
    };
}

group!(Values, 40, "values", 0, Demand::NONE, u32);
group!(Unused, 41, "unused", 0, Demand::NONE, u32);
group!(
    Squares,
    42,
    "squares",
    1,
    Demand::NONE.with(Values::ID),
    u64
);
group!(Parity, 43, "parity", 1, Demand::NONE.with(Values::ID), bool);
group!(
    Summary,
    44,
    "summary",
    2,
    Demand::NONE.with(Squares::ID).with(Parity::ID),
    u64
);

// A group sharing `Squares`' id with another value type (never registered).
group!(Impostor, 42, "impostor", 1, Demand::NONE, i8);

impl AnalysisProducer<[u32]> for Values {
    fn produce(artifact: &[u32], _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        log(Self::NAME);
        (0u32..).zip(artifact.iter().copied()).collect()
    }
}

impl AnalysisProducer<[u32]> for Unused {
    fn produce(_: &[u32], _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        log(Self::NAME);
        AnalysisTable::default()
    }
}

impl AnalysisProducer<[u32]> for Squares {
    fn produce(_: &[u32], inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        log(Self::NAME);
        let values = inputs.get::<Values>().expect("declared input");
        values
            .iter()
            .map(|(key, value)| (*key, u64::from(*value) * u64::from(*value)))
            .collect()
    }
}

impl AnalysisProducer<[u32]> for Parity {
    fn produce(_: &[u32], inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        log(Self::NAME);
        let values = inputs.get::<Values>().expect("declared input");
        values
            .iter()
            .map(|(key, value)| (*key, value % 2 == 0))
            .collect()
    }
}

impl AnalysisProducer<[u32]> for Summary {
    fn produce(_: &[u32], inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        log(Self::NAME);
        let squares = inputs.get::<Squares>().expect("declared input");
        let parity = inputs.get::<Parity>().expect("declared input");
        let even_squares = squares
            .iter()
            .filter(|(key, _)| parity.get(key) == Some(&true))
            .map(|(_, square)| square)
            .sum();
        [(0, even_squares)].into_iter().collect()
    }
}

/// The fixture registry: stratification-checked at compile time.
pub const REGISTRY: AnalysisRegistry<[u32]> = AnalysisRegistry::new(&[
    ProducerEntry::of::<Values>(),
    ProducerEntry::of::<Unused>(),
    ProducerEntry::of::<Squares>(),
    ProducerEntry::of::<Parity>(),
    ProducerEntry::of::<Summary>(),
]);

/// A registry without `Unused`, for the unregistered-demand case.
pub const PARTIAL: AnalysisRegistry<[u32]> =
    AnalysisRegistry::new(&[ProducerEntry::of::<Values>()]);

pub struct SquareRule;
impl AnalysisConsumer for SquareRule {
    const NAME: &'static str = "square-rule";
    const DEMAND: Demand = Demand::NONE.with(Squares::ID);
}

pub struct SummaryRule;
impl AnalysisConsumer for SummaryRule {
    const NAME: &'static str = "summary-rule";
    const DEMAND: Demand = Demand::NONE.with(Summary::ID);
}

pub struct UnusedRule;
impl AnalysisConsumer for UnusedRule {
    const NAME: &'static str = "unused-rule";
    const DEMAND: Demand = Demand::NONE.with(Unused::ID);
}
