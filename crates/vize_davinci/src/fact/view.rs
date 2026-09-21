//! [`FactView`], [`FactError`], the per-artifact table store, and the
//! process-global counters that pin "each group at most once per artifact"
//! and "zero undeclared accesses".

use core::any::{Any, type_name};
use core::sync::atomic::{AtomicU64, Ordering};

use super::registry::ErasedTable;
use super::{Demand, FactGroup, FactTable};
use crate::pass::{AnalysisId, MAX_ANALYSES};

pub(crate) const SLOTS: usize = MAX_ANALYSES as usize;

/// How many times each group's producer has run, process-wide.
static PRODUCED: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];

/// How many undeclared accesses the debug detector has refused, process-wide.
static UNDECLARED: AtomicU64 = AtomicU64::new(0);

/// How many times `group`'s producer has run in this process. A manager runs
/// a producer at most once per artifact state (P4-1b invalidation starts a
/// new state), so across `n` artifacts this grows by at most `n`.
/// Verification recomputes are not productions and are not counted.
#[must_use]
pub fn produced_count(group: AnalysisId) -> u64 {
    PRODUCED[group.index() as usize].load(Ordering::Relaxed)
}

pub(crate) fn count_production(group: AnalysisId) {
    PRODUCED[group.index() as usize].fetch_add(1, Ordering::Relaxed);
}

/// How many undeclared accesses the detector has refused in this process —
/// TS-35's "zero undeclared accesses" reads this. Always 0 in release
/// builds, where the detector does not exist.
#[must_use]
pub fn undeclared_accesses() -> u64 {
    UNDECLARED.load(Ordering::Relaxed)
}

/// Why a fact query, a run or a preservation check failed. Every variant
/// names the group, so a test asserts the exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactError {
    /// A consumer (or producer) read a group outside its declared demand.
    /// Reported by the debug detector only.
    Undeclared {
        consumer: &'static str,
        group: AnalysisId,
    },
    /// A demanded group has no registered producer.
    Unregistered { group: AnalysisId },
    /// The group was not computed for this artifact: no run demanded it, or
    /// a pass invalidated it since.
    NotComputed { group: AnalysisId },
    /// The stored table belongs to another group type with the same id.
    TypeMismatch {
        group: AnalysisId,
        expected: &'static str,
    },
    /// A pass claimed to preserve `group`, but recomputing it on the pass's
    /// output disagrees with the kept table (P4-1b verify mode).
    StalePreserved {
        pass: &'static str,
        group: AnalysisId,
    },
}

/// The tables computed for one artifact.
pub(crate) struct Tables {
    pub(crate) slots: [Option<ErasedTable>; SLOTS],
    pub(crate) computed: Demand,
}

impl Tables {
    pub(crate) fn new() -> Self {
        Self {
            slots: [const { None }; SLOTS],
            computed: Demand::NONE,
        }
    }

    pub(crate) fn slot(&self, group: AnalysisId) -> Option<&(dyn Any + Send + Sync)> {
        self.slots[group.index() as usize].as_deref()
    }

    pub(crate) fn store(&mut self, group: AnalysisId, table: ErasedTable) {
        self.slots[group.index() as usize] = Some(table);
        self.computed = self.computed.with(group);
    }

    /// Drop every table in `groups`.
    pub(crate) fn drop_groups(&mut self, groups: Demand) {
        for group in groups.iter() {
            self.slots[group.index() as usize] = None;
        }
        self.computed = self.computed.minus(groups);
    }
}

/// What the detector knows about the reader: its name and declared demand
/// in debug builds, nothing at all in release builds.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Declared {
    #[cfg(debug_assertions)]
    consumer: &'static str,
    #[cfg(debug_assertions)]
    demand: Demand,
}

// The release shape carries nothing: the detector costs zero bytes and zero
// instructions outside debug builds.
#[cfg(not(debug_assertions))]
const _: () = assert!(size_of::<Declared>() == 0);

impl Declared {
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    pub(crate) const fn new(consumer: &'static str, demand: Demand) -> Self {
        Self {
            #[cfg(debug_assertions)]
            consumer,
            #[cfg(debug_assertions)]
            demand,
        }
    }

    #[cfg_attr(not(debug_assertions), allow(unused_variables, clippy::unused_self))]
    #[inline]
    fn check(self, group: AnalysisId) -> Result<(), FactError> {
        #[cfg(debug_assertions)]
        if !self.demand.contains(group) {
            UNDECLARED.fetch_add(1, Ordering::Relaxed);
            return Err(FactError::Undeclared {
                consumer: self.consumer,
                group,
            });
        }
        Ok(())
    }
}

/// A read-only window onto one artifact's facts, scoped to one reader.
///
/// In debug builds it carries the reader's declared demand and refuses
/// everything outside it; in release builds it is one pointer.
#[derive(Clone, Copy)]
pub struct FactView<'m> {
    tables: &'m Tables,
    declared: Declared,
}

impl<'m> FactView<'m> {
    pub(crate) const fn new(tables: &'m Tables, declared: Declared) -> Self {
        Self { tables, declared }
    }

    /// Group `G`'s table.
    ///
    /// # Errors
    ///
    /// [`FactError::Undeclared`] (debug builds) when `G` is outside the
    /// reader's declared demand; [`FactError::NotComputed`] when no run
    /// demanded `G`; [`FactError::TypeMismatch`] when another group type
    /// owns `G`'s id.
    pub fn get<G: FactGroup>(&self) -> Result<&'m FactTable<G>, FactError> {
        self.declared.check(G::ID)?;
        let table = self
            .tables
            .slot(G::ID)
            .ok_or(FactError::NotComputed { group: G::ID })?;
        table
            .downcast_ref::<FactTable<G>>()
            .ok_or(FactError::TypeMismatch {
                group: G::ID,
                expected: type_name::<G>(),
            })
    }
}
