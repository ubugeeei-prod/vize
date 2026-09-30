//! Fact groups and the fact manager: analyses computed on demand, once.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use core::marker::PhantomData;

use crate::id::AnalysisId;

/// A set of analyses, as a bitmask over [`AnalysisId`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Demand(u64);

impl Demand {
    /// Demands nothing.
    pub const NONE: Self = Self(0);

    /// This demand plus `group`.
    #[must_use]
    pub const fn with(self, group: AnalysisId) -> Self {
        let _ = group;
        todo!()
    }

    /// True when `group` is demanded.
    #[must_use]
    pub const fn contains(self, group: AnalysisId) -> bool {
        let _ = group;
        todo!()
    }
}

/// A group of facts one analysis produces.
pub trait FactGroup: Sized + 'static {
    /// The analysis identity.
    const ID: AnalysisId;
    /// Groups this one reads.
    const DEPENDS_ON: Demand;
    /// What a fact is keyed by.
    type Key;
    /// The fact value.
    type Value;
}

/// Why facts could not be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactError {
    /// A demanded group has no producer registered.
    Unregistered(AnalysisId),
    /// The registry's dependencies form a cycle.
    Cycle(AnalysisId),
}

/// Computes demanded fact groups over one artifact, each at most once.
#[derive(Debug)]
pub struct FactManager<'a, A: ?Sized> {
    computed: Demand,
    _artifact: PhantomData<&'a A>,
}

impl<A: ?Sized> FactManager<'_, A> {
    /// A manager with nothing computed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            computed: Demand::NONE,
            _artifact: PhantomData,
        }
    }

    /// The groups computed so far.
    #[must_use]
    pub const fn computed(&self) -> Demand {
        self.computed
    }

    /// Compute `demand` (and its dependencies) over `artifact`.
    pub fn compute(&mut self, artifact: &A, demand: Demand) -> Result<Demand, FactError> {
        let _ = (artifact, demand);
        todo!()
    }
}

impl<A: ?Sized> Default for FactManager<'_, A> {
    fn default() -> Self {
        Self::new()
    }
}
