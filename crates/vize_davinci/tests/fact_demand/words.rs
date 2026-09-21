//! Fixture fact groups over a word list, with consumers that read exactly
//! what they declare and two that do not.
//!
//! ```text
//! stratum 0   lengths            vowels
//! stratum 1   longest(lengths)   leaky(lengths; reads vowels undeclared)
//! ```

use std::sync::{Mutex, MutexGuard};

use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry,
};
use vize_davinci::pass::AnalysisId;

/// Serializes the tests that read the process-global detector counter.
static LOCK: Mutex<()> = Mutex::new(());

pub fn serial() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub type Words = [&'static str];

pub struct Lengths;
impl FactGroup for Lengths {
    const ID: AnalysisId = AnalysisId::new(20);
    const NAME: &'static str = "lengths";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = usize;
}
impl FactProducer<Words> for Lengths {
    fn produce(words: &Words, _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(words.iter().map(|word| word.len())).collect()
    }
}

pub struct Vowels;
impl FactGroup for Vowels {
    const ID: AnalysisId = AnalysisId::new(21);
    const NAME: &'static str = "vowels";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = usize;
}
impl FactProducer<Words> for Vowels {
    fn produce(words: &Words, _: &FactView<'_>) -> FactTable<Self> {
        let vowels = |word: &str| word.chars().filter(|c| "aeiou".contains(*c)).count();
        (0u32..)
            .zip(words.iter().map(|word| vowels(word)))
            .collect()
    }
}

pub struct Longest;
impl FactGroup for Longest {
    const ID: AnalysisId = AnalysisId::new(22);
    const NAME: &'static str = "longest";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Lengths::ID);
    type Key = ();
    type Value = u32;
}
impl FactProducer<Words> for Longest {
    fn produce(_: &Words, inputs: &FactView<'_>) -> FactTable<Self> {
        let lengths = inputs.get::<Lengths>().expect("declared input");
        let longest = lengths.iter().max_by_key(|(_, len)| **len);
        longest.map(|(at, _)| ((), *at)).into_iter().collect()
    }
}

/// Declares `Lengths` but reads `Vowels`; records what its view answered.
pub struct Leaky;
impl FactGroup for Leaky {
    const ID: AnalysisId = AnalysisId::new(23);
    const NAME: &'static str = "leaky";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Lengths::ID);
    type Key = ();
    type Value = vize_s0::String;
}
impl FactProducer<Words> for Leaky {
    fn produce(_: &Words, inputs: &FactView<'_>) -> FactTable<Self> {
        let seen = inputs.get::<Vowels>().map(|table| table.len());
        [((), vize_s0::cstr!("{seen:?}"))].into_iter().collect()
    }
}

/// Stratification-checked at compile time.
pub const REGISTRY: FactRegistry<Words> = FactRegistry::new(&[
    ProducerEntry::of::<Lengths>(),
    ProducerEntry::of::<Vowels>(),
    ProducerEntry::of::<Longest>(),
    ProducerEntry::of::<Leaky>(),
]);

pub struct LongestRule;
impl FactConsumer for LongestRule {
    const NAME: &'static str = "longest-rule";
    const DEMAND: Demand = Demand::NONE.with(Longest::ID);
}

pub struct VowelRule;
impl FactConsumer for VowelRule {
    const NAME: &'static str = "vowel-rule";
    const DEMAND: Demand = Demand::NONE.with(Vowels::ID);
}

/// Declares `Longest` but also reads `Vowels`.
pub struct SneakyRule;
impl FactConsumer for SneakyRule {
    const NAME: &'static str = "sneaky-rule";
    const DEMAND: Demand = Demand::NONE.with(Longest::ID);
}

pub struct LeakyRule;
impl FactConsumer for LeakyRule {
    const NAME: &'static str = "leaky-rule";
    const DEMAND: Demand = Demand::NONE.with(Leaky::ID);
}
