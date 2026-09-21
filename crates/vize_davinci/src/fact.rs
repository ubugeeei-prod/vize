//! The fact API — the one typed query surface of charter #5/#8 (P4-1a).
//!
//! Every analysis a consumer reads is a **fact group**: a table keyed by
//! [`FactGroup::Key`], computed by a registered producer, never a field on a
//! god struct. Consumers **declare** the groups they read as const data
//! ([`FactConsumer::DEMAND`]); a run computes exactly the transitive closure of
//! the declared union, in stratum order, each group at most once per artifact
//! ([`FactManager`]).
//!
//! # Identity
//!
//! A group's identity is a pass-manager [`AnalysisId`], so a pass's
//! [`Preserved`](crate::pass::Preserved) mask names fact groups directly: one
//! identity space, capped by [`MAX_ANALYSES`](crate::pass::MAX_ANALYSES) and
//! its existing const assertion. A [`Demand`] is the same 64-bit shape.
//!
//! # Stratification — demand cycles are unrepresentable
//!
//! A group names a [`FactGroup::STRATUM`] and may depend only on groups in
//! **strictly lower** strata. [`FactRegistry::new`] checks that in a `const
//! fn`, so a registry that breaks it is a compile error; with every edge
//! pointing strictly downwards no cycle can be written down at all (the Swift
//! request-evaluator anti-lesson: cycle detection at run time is a design
//! that already failed).
//!
//! # The undeclared-access detector (TS-35)
//!
//! Under `debug_assertions` a [`FactView`] carries its consumer's declared
//! demand; reading a group outside it returns the exact
//! [`FactError::Undeclared`] and bumps a process-global counter
//! ([`undeclared_accesses`]) even when another consumer's demand happened to
//! compute the table. In release builds the declared demand is a zero-sized
//! type (const-asserted) and the check compiles away.
//!
//! # Example
//!
//! ```
//! use vize_davinci::fact::{
//!     Demand, FactConsumer, FactGroup, FactManager, FactProducer, FactRegistry, FactTable,
//!     FactView, ProducerEntry,
//! };
//! use vize_davinci::pass::AnalysisId;
//!
//! /// Stratum 0: the length of every word.
//! struct Lengths;
//! impl FactGroup for Lengths {
//!     const ID: AnalysisId = AnalysisId::new(0);
//!     const NAME: &'static str = "lengths";
//!     const STRATUM: u8 = 0;
//!     const DEPENDS: Demand = Demand::NONE;
//!     type Key = u32;
//!     type Value = usize;
//! }
//! impl FactProducer<[&str]> for Lengths {
//!     fn produce(words: &[&str], _: &FactView<'_>) -> FactTable<Self> {
//!         (0u32..).zip(words.iter().map(|word| word.len())).collect()
//!     }
//! }
//!
//! /// Stratum 1: the longest word, read off `Lengths`.
//! struct Longest;
//! impl FactGroup for Longest {
//!     const ID: AnalysisId = AnalysisId::new(1);
//!     const NAME: &'static str = "longest";
//!     const STRATUM: u8 = 1;
//!     const DEPENDS: Demand = Demand::NONE.with(Lengths::ID);
//!     type Key = ();
//!     type Value = u32;
//! }
//! impl FactProducer<[&str]> for Longest {
//!     fn produce(_: &[&str], inputs: &FactView<'_>) -> FactTable<Self> {
//!         let lengths = inputs.get::<Lengths>().expect("declared");
//!         let longest = lengths.iter().max_by_key(|(_, len)| **len).map(|(at, _)| *at);
//!         longest.map(|at| ((), at)).into_iter().collect()
//!     }
//! }
//!
//! const REGISTRY: FactRegistry<[&str]> =
//!     FactRegistry::new(&[ProducerEntry::of::<Lengths>(), ProducerEntry::of::<Longest>()]);
//!
//! struct LongestWordRule;
//! impl FactConsumer for LongestWordRule {
//!     const NAME: &'static str = "longest-word";
//!     const DEMAND: Demand = Demand::NONE.with(Longest::ID);
//! }
//!
//! let words: &[&str] = &["fact", "demand", "view"];
//! let mut manager = FactManager::new(&REGISTRY, words);
//! let view = manager.prepare::<LongestWordRule>().unwrap();
//! assert_eq!(view.get::<Longest>().unwrap().get(&()), Some(&1));
//! ```
//!
//! # Module layout
//!
//! - [`demand`] — [`Demand`], the const-built group set
//! - [`registry`] — [`GroupDesc`], [`FactRegistry`] and the stratification
//!   check
//! - [`table`] — [`FactTable`], the borrowed table a query returns
//! - [`manager`] — [`FactManager`], [`FactView`], [`FactError`] and the
//!   process-global counters

pub mod demand;
pub mod manager;
pub mod registry;
pub mod table;

pub use demand::Demand;
pub use manager::{FactError, FactManager, FactView, produced_count, undeclared_accesses};
pub use registry::{FactRegistry, GroupDesc, ProducerEntry, StrataError, check_strata};
pub use table::{FactTable, FactTableBuilder};

use crate::pass::AnalysisId;

/// One fact group: its identity, its stratum, what it reads, and the shape of
/// its table.
///
/// Implemented by a unit type per group. Everything is `const` so the
/// registry can check stratification at compile time and a consumer can
/// build its demand in a `const` item.
pub trait FactGroup: Sized + 'static {
    /// The group's identity — shared with pass-manager preserved masks.
    const ID: AnalysisId;
    /// The group's stable name, used in errors and dumps.
    const NAME: &'static str;
    /// The group's stratum. Every group in [`FactGroup::DEPENDS`] must sit in
    /// a strictly lower one.
    const STRATUM: u8;
    /// The groups this group's producer reads.
    const DEPENDS: Demand;
    /// The key facts are stored under (`NodeId`, `SymbolId`, an artifact key).
    type Key: Ord + Send + Sync + 'static;
    /// One fact.
    type Value: Send + Sync + 'static;

    /// This group as const registry data.
    const DESC: GroupDesc = GroupDesc {
        id: Self::ID,
        name: Self::NAME,
        stratum: Self::STRATUM,
        depends: Self::DEPENDS,
    };
}

/// A consumer of facts: a lint rule, a lowering, a projection, an LSP
/// feature. Its demand is const data, declared once, read by the detector.
pub trait FactConsumer {
    /// The consumer's stable name, reported by [`FactError::Undeclared`].
    const NAME: &'static str;
    /// Every group the consumer reads.
    const DEMAND: Demand;
}

/// How a registered producer computes its group over an artifact `A`.
///
/// The producer reads its inputs through a [`FactView`] whose declared
/// demand is its own [`FactGroup::DEPENDS`], so a producer reading a group it
/// did not declare trips the same detector a consumer does.
pub trait FactProducer<A: ?Sized>: FactGroup {
    /// Compute the whole table for `artifact`.
    fn produce(artifact: &A, inputs: &FactView<'_>) -> FactTable<Self>;
}
