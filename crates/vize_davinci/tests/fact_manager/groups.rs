//! Fixture fact groups over a `[u32]` artifact.
//!
//! ```text
//! stratum 0   values            unused (registered, never demanded)
//! stratum 1   squares(values)   parity(values)
//! stratum 2   summary(squares, parity)
//! ```

use std::sync::{Mutex, MutexGuard};

use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry,
};
use vize_davinci::pass::AnalysisId;

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
        impl FactGroup for $ty {
            const ID: AnalysisId = AnalysisId::new($id);
            const NAME: &'static str = $name;
            const STRATUM: u8 = $stratum;
            const DEPENDS: Demand = $depends;
            type Key = u32;
            type Value = $value;
        }
    };
}

group!(Values, 10, "values", 0, Demand::NONE, u32);
group!(Unused, 11, "unused", 0, Demand::NONE, u32);
group!(
    Squares,
    12,
    "squares",
    1,
    Demand::NONE.with(Values::ID),
    u64
);
group!(Parity, 13, "parity", 1, Demand::NONE.with(Values::ID), bool);
group!(
    Summary,
    14,
    "summary",
    2,
    Demand::NONE.with(Squares::ID).with(Parity::ID),
    u64
);

// A group sharing `Squares`' id with another value type (never registered).
group!(Impostor, 12, "impostor", 1, Demand::NONE, i8);

impl FactProducer<[u32]> for Values {
    fn produce(artifact: &[u32], _: &FactView<'_>) -> FactTable<Self> {
        log(Self::NAME);
        (0u32..).zip(artifact.iter().copied()).collect()
    }
}

impl FactProducer<[u32]> for Unused {
    fn produce(_: &[u32], _: &FactView<'_>) -> FactTable<Self> {
        log(Self::NAME);
        FactTable::default()
    }
}

impl FactProducer<[u32]> for Squares {
    fn produce(_: &[u32], inputs: &FactView<'_>) -> FactTable<Self> {
        log(Self::NAME);
        let values = inputs.get::<Values>().expect("declared input");
        values
            .iter()
            .map(|(key, value)| (*key, u64::from(*value) * u64::from(*value)))
            .collect()
    }
}

impl FactProducer<[u32]> for Parity {
    fn produce(_: &[u32], inputs: &FactView<'_>) -> FactTable<Self> {
        log(Self::NAME);
        let values = inputs.get::<Values>().expect("declared input");
        values
            .iter()
            .map(|(key, value)| (*key, value % 2 == 0))
            .collect()
    }
}

impl FactProducer<[u32]> for Summary {
    fn produce(_: &[u32], inputs: &FactView<'_>) -> FactTable<Self> {
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
pub const REGISTRY: FactRegistry<[u32]> = FactRegistry::new(&[
    ProducerEntry::of::<Values>(),
    ProducerEntry::of::<Unused>(),
    ProducerEntry::of::<Squares>(),
    ProducerEntry::of::<Parity>(),
    ProducerEntry::of::<Summary>(),
]);

/// A registry without `Unused`, for the unregistered-demand case.
pub const PARTIAL: FactRegistry<[u32]> = FactRegistry::new(&[ProducerEntry::of::<Values>()]);

pub struct SquareRule;
impl FactConsumer for SquareRule {
    const NAME: &'static str = "square-rule";
    const DEMAND: Demand = Demand::NONE.with(Squares::ID);
}

pub struct SummaryRule;
impl FactConsumer for SummaryRule {
    const NAME: &'static str = "summary-rule";
    const DEMAND: Demand = Demand::NONE.with(Summary::ID);
}

pub struct UnusedRule;
impl FactConsumer for UnusedRule {
    const NAME: &'static str = "unused-rule";
    const DEMAND: Demand = Demand::NONE.with(Unused::ID);
}
