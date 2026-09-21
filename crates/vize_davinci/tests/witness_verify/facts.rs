//! Fixture fact groups over a `str` artifact, and the rule that cites them.
//!
//! ```text
//! stratum 0   words          sentence       lengths (not witness-capable)   hidden
//! stratum 1   repeats(words)
//! ```
//!
//! `repeated-word` demands `words`, `repeats` and `sentence`; `hidden` is
//! witness-capable but outside its demand.

use vize_davinci::diagnostic::{Diagnostic, Stage, Verdict, WitnessChain, WitnessLink};
use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry,
};
use vize_davinci::pass::AnalysisId;
use vize_davinci::witness::{WitnessCheck, WitnessChecks, WitnessGroup};
use vize_s0::{Span, String, cstr};

/// One word: where it is, and whether it is a proof (a trailing `?` makes
/// the word a maybe).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub span: Span,
    pub text: String,
    pub verdict: Verdict,
}

macro_rules! group {
    ($ty:ident, $id:expr, $name:literal, $stratum:expr, $depends:expr, $key:ty, $value:ty) => {
        pub struct $ty;
        impl FactGroup for $ty {
            const ID: AnalysisId = AnalysisId::new($id);
            const NAME: &'static str = $name;
            const STRATUM: u8 = $stratum;
            const DEPENDS: Demand = $depends;
            type Key = $key;
            type Value = $value;
        }
    };
}

group!(Words, 20, "words", 0, Demand::NONE, u32, Word);
group!(
    Repeats,
    21,
    "repeats",
    1,
    Demand::NONE.with(Words::ID),
    String,
    Span
);
group!(Sentence, 22, "sentence", 0, Demand::NONE, (), Span);
group!(Lengths, 23, "lengths", 0, Demand::NONE, u32, usize);
group!(Hidden, 24, "hidden", 0, Demand::NONE, u32, Span);

fn words(text: &str) -> impl Iterator<Item = (u32, Word)> + '_ {
    let mut offset = 0u32;
    let mut index = 0u32;
    text.split(' ').filter_map(move |piece| {
        let start = offset;
        offset += u32::try_from(piece.len()).expect("fixture text fits u32") + 1;
        if piece.is_empty() {
            return None;
        }
        let verdict = if piece.ends_with('?') {
            Verdict::Unknown
        } else {
            Verdict::Proven
        };
        let end = start + u32::try_from(piece.len()).expect("fits");
        let word = Word {
            span: Span::new(start, end),
            text: String::from(piece.trim_end_matches('?')),
            verdict,
        };
        index += 1;
        Some((index - 1, word))
    })
}

impl FactProducer<str> for Words {
    fn produce(text: &str, _: &FactView<'_>) -> FactTable<Self> {
        words(text).collect()
    }
}

impl FactProducer<str> for Repeats {
    fn produce(_: &str, inputs: &FactView<'_>) -> FactTable<Self> {
        let words = inputs.get::<Words>().expect("declared input");
        let mut repeats = Vec::new();
        for (index, word) in words.iter() {
            let earlier = words.iter().take_while(|(at, _)| *at < index);
            if earlier.filter(|(_, seen)| seen.text == word.text).count() == 1 {
                repeats.push((word.text.clone(), word.span));
            }
        }
        repeats.into_iter().collect()
    }
}

impl FactProducer<str> for Sentence {
    fn produce(text: &str, _: &FactView<'_>) -> FactTable<Self> {
        let length = u32::try_from(text.len()).expect("fits");
        [((), Span::new(0, length))].into_iter().collect()
    }
}

impl FactProducer<str> for Lengths {
    fn produce(text: &str, _: &FactView<'_>) -> FactTable<Self> {
        words(text)
            .map(|(index, word)| (index, word.text.len()))
            .collect()
    }
}

impl FactProducer<str> for Hidden {
    fn produce(text: &str, _: &FactView<'_>) -> FactTable<Self> {
        words(text)
            .map(|(index, word)| (index, word.span))
            .collect()
    }
}

impl WitnessGroup for Words {
    fn fact_span(_: &u32, word: &Word) -> Span {
        word.span
    }

    fn verdict(word: &Word) -> Verdict {
        word.verdict
    }
}

impl WitnessGroup for Repeats {
    fn fact_span(_: &String, second: &Span) -> Span {
        *second
    }
}

impl WitnessGroup for Sentence {
    fn fact_span(_: &(), whole: &Span) -> Span {
        *whole
    }
}

impl WitnessGroup for Hidden {
    fn fact_span(_: &u32, span: &Span) -> Span {
        *span
    }
}

pub static REGISTRY: FactRegistry<str> = FactRegistry::new(&[
    ProducerEntry::of::<Words>(),
    ProducerEntry::of::<Sentence>(),
    ProducerEntry::of::<Lengths>(),
    ProducerEntry::of::<Hidden>(),
    ProducerEntry::of::<Repeats>(),
]);

pub static CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<Words>(),
    WitnessCheck::of::<Repeats>(),
    WitnessCheck::of::<Sentence>(),
    WitnessCheck::of::<Hidden>(),
]);

/// The rule: an error on every word's second occurrence, proved by the
/// first occurrence, the repeat fact and the sentence it sits in. It fires
/// on proof only — a repeat whose first occurrence is a maybe is silent.
pub struct RepeatedWord;

impl FactConsumer for RepeatedWord {
    const NAME: &'static str = "repeated-word";
    const DEMAND: Demand = Demand::NONE
        .with(Words::ID)
        .with(Repeats::ID)
        .with(Sentence::ID);
}

pub fn repeated_words(facts: &FactView<'_>) -> Vec<Diagnostic> {
    let words = facts.get::<Words>().expect("declared");
    let repeats = facts.get::<Repeats>().expect("declared");
    let sentence = facts.get::<Sentence>().expect("declared");
    let whole = *sentence.get(&()).expect("one sentence");
    let mut found = Vec::new();
    for (text, second) in repeats.iter() {
        let (first_index, first) = words
            .iter()
            .find(|(_, word)| word.text == *text)
            .expect("a repeat has a first occurrence");
        if !first.verdict.is_proven() {
            continue;
        }
        let chain = WitnessChain::new(WitnessLink::of::<Words>(first_index, first.span))
            .then(WitnessLink::of::<Repeats>(text, *second))
            .then(WitnessLink::of::<Sentence>(&(), whole));
        let message = cstr!("`{text}` is repeated");
        found.push(Diagnostic::proven(Stage::Semantic, *second, message, chain));
    }
    found
}
