//! Fixture groups over a mutable number list, the passes that rewrite it,
//! and two groups at production ids for the named preservation groups.
//!
//! ```text
//! stratum 0   values   count   bindings-like   usages-like
//! stratum 1   total(values)
//! ```

use std::sync::{Mutex, MutexGuard};

use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView,
    ProducerEntry, ids,
};
use vize_davinci::pass::{AnalysisId, Fusability, PassDesc, PassKind, Preserved};

/// Serializes the tests that read the process-global production counter.
static LOCK: Mutex<()> = Mutex::new(());

pub fn serial() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub type Numbers = Vec<u32>;

macro_rules! group {
    ($ty:ident, $id:expr, $name:literal, $stratum:expr, $depends:expr, $key:ty, $value:ty) => {
        pub struct $ty;
        impl FactGroup for $ty {
            const ID: AnalysisId = $id;
            const NAME: &'static str = $name;
            const STRATUM: u8 = $stratum;
            const DEPENDS: Demand = $depends;
            type Key = $key;
            type Value = $value;
        }
    };
}

group!(
    Values,
    AnalysisId::new(32),
    "values",
    0,
    Demand::NONE,
    u32,
    u32
);
group!(
    Count,
    AnalysisId::new(33),
    "count",
    0,
    Demand::NONE,
    (),
    usize
);
group!(
    Total,
    AnalysisId::new(34),
    "total",
    1,
    Demand::NONE.with(Values::ID),
    (),
    u64
);
group!(
    BindingsLike,
    ids::BINDINGS,
    "bindings-like",
    0,
    Demand::NONE,
    (),
    usize
);
group!(
    UsagesLike,
    ids::COMPONENT_USAGES,
    "usages-like",
    0,
    Demand::NONE,
    (),
    usize
);

impl FactProducer<Numbers> for Values {
    fn produce(numbers: &Numbers, _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(numbers.iter().copied()).collect()
    }
}

impl FactProducer<Numbers> for Count {
    fn produce(numbers: &Numbers, _: &FactView<'_>) -> FactTable<Self> {
        [((), numbers.len())].into_iter().collect()
    }
}

impl FactProducer<Numbers> for Total {
    fn produce(_: &Numbers, inputs: &FactView<'_>) -> FactTable<Self> {
        let values = inputs.get::<Values>().expect("declared input");
        let total = values.iter().map(|(_, value)| u64::from(*value)).sum();
        [((), total)].into_iter().collect()
    }
}

impl FactProducer<Numbers> for BindingsLike {
    fn produce(numbers: &Numbers, _: &FactView<'_>) -> FactTable<Self> {
        [((), numbers.len())].into_iter().collect()
    }
}

impl FactProducer<Numbers> for UsagesLike {
    fn produce(numbers: &Numbers, _: &FactView<'_>) -> FactTable<Self> {
        [((), numbers.len())].into_iter().collect()
    }
}

pub const REGISTRY: FactRegistry<Numbers> = FactRegistry::new(&[
    ProducerEntry::of::<Values>(),
    ProducerEntry::of::<Count>(),
    ProducerEntry::of::<Total>(),
    ProducerEntry::of::<BindingsLike>(),
    ProducerEntry::of::<UsagesLike>(),
]);

/// Reads everything the fixture computes.
pub struct Everything;
impl FactConsumer for Everything {
    const NAME: &'static str = "everything";
    const DEMAND: Demand = Demand::NONE
        .with(Values::ID)
        .with(Count::ID)
        .with(Total::ID)
        .with(BindingsLike::ID)
        .with(UsagesLike::ID);
}

/// Changes nothing and says so.
pub const RENAME: PassDesc = PassDesc::new(
    "rename",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);

/// Doubles every value; honestly preserves only what does not read values.
pub const DOUBLE: PassDesc = PassDesc::new(
    "double",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::NONE
        .with(Count::ID)
        .with(BindingsLike::ID)
        .with(UsagesLike::ID),
);

/// Doubles every value but claims to preserve everything.
pub const LYING_DOUBLE: PassDesc = PassDesc::new(
    "lying-double",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);

/// The body `DOUBLE` and `LYING_DOUBLE` share.
pub fn double(numbers: &mut Numbers) {
    for value in numbers.iter_mut() {
        *value *= 2;
    }
}
