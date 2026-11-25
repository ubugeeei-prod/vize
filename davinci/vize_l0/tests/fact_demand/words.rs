//! Fixture fact groups over a word list, with consumers that read exactly
//! what they declare and two that do not.
//!
//! ```text
//! stratum 0   lengths            vowels
//! stratum 1   longest(lengths)   leaky(lengths; reads vowels undeclared)
//! ```

use std::sync::{Mutex, MutexGuard};

use vize_l0::analysis::{
    Analysis, AnalysisConsumer, AnalysisProducer, AnalysisRegistry, AnalysisTable, AnalysisView,
    Demand, ProducerEntry,
};
use vize_l0::pass::AnalysisId;

/// Serializes the tests that read the process-global detector counter.
static LOCK: Mutex<()> = Mutex::new(());

pub fn serial() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub type Words = [&'static str];

pub struct Lengths;
impl Analysis for Lengths {
    const ID: AnalysisId = AnalysisId::new(50);
    const NAME: &'static str = "lengths";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = usize;
}
impl AnalysisProducer<Words> for Lengths {
    fn produce(words: &Words, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        (0u32..).zip(words.iter().map(|word| word.len())).collect()
    }
}

pub struct Vowels;
impl Analysis for Vowels {
    const ID: AnalysisId = AnalysisId::new(51);
    const NAME: &'static str = "vowels";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = usize;
}
impl AnalysisProducer<Words> for Vowels {
    fn produce(words: &Words, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let vowels = |word: &str| word.chars().filter(|c| "aeiou".contains(*c)).count();
        (0u32..)
            .zip(words.iter().map(|word| vowels(word)))
            .collect()
    }
}

pub struct Longest;
impl Analysis for Longest {
    const ID: AnalysisId = AnalysisId::new(52);
    const NAME: &'static str = "longest";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Lengths::ID);
    type Key = ();
    type Value = u32;
}
impl AnalysisProducer<Words> for Longest {
    fn produce(_: &Words, inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let lengths = inputs.get::<Lengths>().expect("declared input");
        let longest = lengths.iter().max_by_key(|(_, len)| **len);
        longest.map(|(at, _)| ((), *at)).into_iter().collect()
    }
}

/// Declares `Lengths` but reads `Vowels`; records what its view answered.
pub struct Leaky;
impl Analysis for Leaky {
    const ID: AnalysisId = AnalysisId::new(53);
    const NAME: &'static str = "leaky";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Lengths::ID);
    type Key = ();
    type Value = vize_l0::String;
}
impl AnalysisProducer<Words> for Leaky {
    fn produce(_: &Words, inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let seen = inputs.get::<Vowels>().map(|table| table.len());
        [((), vize_l0::cstr!("{seen:?}"))].into_iter().collect()
    }
}

/// Stratification-checked at compile time.
pub const REGISTRY: AnalysisRegistry<Words> = AnalysisRegistry::new(&[
    ProducerEntry::of::<Lengths>(),
    ProducerEntry::of::<Vowels>(),
    ProducerEntry::of::<Longest>(),
    ProducerEntry::of::<Leaky>(),
]);

pub struct LongestRule;
impl AnalysisConsumer for LongestRule {
    const NAME: &'static str = "longest-rule";
    const DEMAND: Demand = Demand::NONE.with(Longest::ID);
}

pub struct VowelRule;
impl AnalysisConsumer for VowelRule {
    const NAME: &'static str = "vowel-rule";
    const DEMAND: Demand = Demand::NONE.with(Vowels::ID);
}

/// Declares `Longest` but also reads `Vowels`.
pub struct SneakyRule;
impl AnalysisConsumer for SneakyRule {
    const NAME: &'static str = "sneaky-rule";
    const DEMAND: Demand = Demand::NONE.with(Longest::ID);
}

pub struct LeakyRule;
impl AnalysisConsumer for LeakyRule {
    const NAME: &'static str = "leaky-rule";
    const DEMAND: Demand = Demand::NONE.with(Leaky::ID);
}
