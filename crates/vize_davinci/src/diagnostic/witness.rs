//! The proof a diagnostic rests on.
//!
//! `assurance.md`: "An error must carry its witness — the concrete fact chain
//! that proves the violation (binding → escape → effect edge, with spans via
//! provenance). The witness is machine-checkable against the fact base, so a
//! false positive is not a matter of opinion: it is a witness that fails
//! verification."
//!
//! A witness is a non-empty [`WitnessChain`] of [`WitnessLink`]s, each naming
//! one fact: its fact group (an [`AnalysisId`], the identity the fact API and
//! the pass manager's preserved masks share), the span the fact is about, and
//! the key it is stored under. P4-6b's verifier re-checks every link against
//! the fact base; this module only makes the chain's shape a type.
//!
//! Producers that predate the witness SDK report errors under a declared
//! [`Exemption`] instead — [`Witness::LegacyExempt`] — so the exemption is an
//! entry in an inventory (`docs/davinci/plan/witness-exemptions.tsv`) rather
//! than an absence, which is the whole point of "never silently".

pub mod exemption;
pub mod key;

use alloc::boxed::Box;
use alloc::vec::Vec;

pub use exemption::Exemption;
pub use key::{WitnessKey, WitnessKeyed};

use crate::fact::FactGroup;
use crate::pass::AnalysisId;
use vize_s0::Span;

/// One link of a witness chain: the fact stored under `key` in fact group
/// `group`, about the source range `span`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WitnessLink {
    /// The fact group the fact lives in.
    pub group: AnalysisId,
    /// The authored-file range the fact is about.
    pub span: Span,
    /// The key the fact is stored under in its group's table.
    pub key: WitnessKey,
}

impl WitnessLink {
    /// A link to the fact under `key` in `group`, about `span`.
    #[must_use]
    pub fn new(group: AnalysisId, span: Span, key: WitnessKey) -> Self {
        Self { group, span, key }
    }

    /// A link to the fact stored under `key` in fact group `G`, about
    /// `span` — the typed form a producer holding `G`'s table writes, so the
    /// group id and the key shape cannot disagree with the table the link
    /// cites.
    #[must_use]
    pub fn of<G: FactGroup<Key: WitnessKeyed>>(key: &G::Key, span: Span) -> Self {
        Self::new(G::ID, span, key.to_witness_key())
    }
}

/// A non-empty chain of [`WitnessLink`]s, in proof order.
///
/// Non-emptiness is the type's invariant: every constructor takes a first
/// link or refuses an empty list, so "an error with an empty proof" is as
/// unrepresentable as "an error with no proof". Stored as a boxed slice so a
/// witness costs the diagnostic 16 bytes, not a `Vec`'s 24.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WitnessChain {
    links: Box<[WitnessLink]>,
}

impl WitnessChain {
    /// A chain of one link.
    #[must_use]
    pub fn new(first: WitnessLink) -> Self {
        Self {
            links: Box::new([first]),
        }
    }

    /// This chain with `link` appended.
    #[must_use]
    pub fn then(self, link: WitnessLink) -> Self {
        let mut links = Vec::from(self.links);
        links.push(link);
        Self {
            links: links.into_boxed_slice(),
        }
    }

    /// The chain of `links`, or `None` when there are none.
    #[must_use]
    pub fn from_links(links: Vec<WitnessLink>) -> Option<Self> {
        (!links.is_empty()).then(|| Self {
            links: links.into_boxed_slice(),
        })
    }

    /// Every link, in proof order. Never empty.
    #[must_use]
    pub fn links(&self) -> &[WitnessLink] {
        &self.links
    }

    /// The first link — the fact the proof starts from.
    #[must_use]
    pub fn first(&self) -> &WitnessLink {
        // The invariant is that `links` is never empty; indexing states it.
        &self.links[0]
    }
}

/// The proof a diagnostic rests on, or the counted exemption standing in for
/// one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Witness {
    /// The fact chain that proves the diagnostic.
    Proven(WitnessChain),
    /// A producer that predates the witness SDK, exempt by inventory.
    ///
    /// The payload is a declared `static`, so the exemption is an entry in a
    /// list rather than an absence.
    LegacyExempt(&'static Exemption),
}
