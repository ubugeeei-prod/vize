//! Fixture groups with α forms over a word list: a keyed number table, a
//! keyed text table and a single scalar.

use vize_l0::analysis::{
    AlphaExport, Analysis, AnalysisConsumer, AnalysisProducer, AnalysisRegistry, AnalysisTable,
    AnalysisView, Demand, ProducerEntry,
};
use vize_l0::dump::Dump;
use vize_l0::pass::AnalysisId;
use vize_l0::{FxHashMap, String};

pub type Words = [&'static str];

pub struct Lengths;
impl Analysis for Lengths {
    const ID: AnalysisId = AnalysisId::new(32);
    const NAME: &'static str = "lengths";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = u32;
}
impl AnalysisProducer<Words> for Lengths {
    fn produce(words: &Words, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        (0u32..)
            .zip(words.iter().map(|word| word.len() as u32))
            .collect()
    }
}

/// The α page of `lengths`: word index → length.
#[derive(Debug, Default, PartialEq, Dump)]
pub struct LengthsAlpha {
    pub lengths: FxHashMap<u32, u32>,
}

impl AlphaExport for Lengths {
    const ALPHA_SCHEMA: u16 = 1;
    type Alpha = LengthsAlpha;
    fn export(table: &AnalysisTable<Self>) -> LengthsAlpha {
        LengthsAlpha {
            lengths: table.iter().map(|(key, value)| (*key, *value)).collect(),
        }
    }
    fn import(alpha: LengthsAlpha) -> AnalysisTable<Self> {
        alpha.lengths.into_iter().collect()
    }
}

pub struct Words2;
impl Analysis for Words2 {
    const ID: AnalysisId = AnalysisId::new(33);
    const NAME: &'static str = "words";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = String;
}
impl AnalysisProducer<Words> for Words2 {
    fn produce(words: &Words, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        (0u32..)
            .zip(words.iter().map(|word| String::from(*word)))
            .collect()
    }
}

/// The α page of `words`: word index → the word.
#[derive(Debug, Default, PartialEq, Dump)]
pub struct WordsAlpha {
    pub words: FxHashMap<u32, String>,
}

impl AlphaExport for Words2 {
    const ALPHA_SCHEMA: u16 = 3;
    type Alpha = WordsAlpha;
    fn export(table: &AnalysisTable<Self>) -> WordsAlpha {
        WordsAlpha {
            words: table
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
        }
    }
    fn import(alpha: WordsAlpha) -> AnalysisTable<Self> {
        alpha.words.into_iter().collect()
    }
}

pub struct Longest;
impl Analysis for Longest {
    const ID: AnalysisId = AnalysisId::new(34);
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

/// The α page of `longest`: the index of the longest word, when any.
#[derive(Debug, Default, PartialEq, Dump)]
pub struct LongestAlpha {
    pub present: bool,
    pub index: u32,
}

impl AlphaExport for Longest {
    const ALPHA_SCHEMA: u16 = 1;
    type Alpha = LongestAlpha;
    fn export(table: &AnalysisTable<Self>) -> LongestAlpha {
        match table.get(&()) {
            Some(index) => LongestAlpha {
                present: true,
                index: *index,
            },
            None => LongestAlpha::default(),
        }
    }
    fn import(alpha: LongestAlpha) -> AnalysisTable<Self> {
        alpha
            .present
            .then_some(((), alpha.index))
            .into_iter()
            .collect()
    }
}

pub const REGISTRY: AnalysisRegistry<Words> = AnalysisRegistry::new(&[
    ProducerEntry::of::<Lengths>(),
    ProducerEntry::of::<Words2>(),
    ProducerEntry::of::<Longest>(),
]);

pub struct Summary;
impl AnalysisConsumer for Summary {
    const NAME: &'static str = "summary";
    const DEMAND: Demand = Demand::NONE
        .with(Lengths::ID)
        .with(Words2::ID)
        .with(Longest::ID);
}
