//! The pass manager: pipelines as const data, fused into single walks.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use crate::id::AnalysisId;
use crate::level::Level;

pub mod observer;

use observer::PassObserver;

/// How many analyses one [`Preserved`] set tracks.
pub const MAX_ANALYSES: u8 = 64;

/// The analyses a pass leaves valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preserved(u64);

impl Preserved {
    /// Preserves nothing.
    pub const NOTHING: Self = Self(0);

    /// This set plus `analysis`.
    #[must_use]
    pub const fn with(self, analysis: AnalysisId) -> Self {
        let _ = analysis;
        todo!()
    }

    /// True when `analysis` stays valid.
    #[must_use]
    pub const fn preserves(self, analysis: AnalysisId) -> bool {
        let _ = analysis;
        todo!()
    }

    /// What both sets preserve.
    #[must_use]
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

/// A pass's static description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassDesc {
    /// Pipeline name.
    pub name: &'static str,
    /// The level the pass runs on.
    pub level: Level,
    /// Mandatory passes are fusion barriers.
    pub mandatory: bool,
    /// Analyses left valid.
    pub preserves: Preserved,
}

/// A pass: its description is const data.
pub trait Pass {
    /// The static description.
    const DESC: PassDesc;
}

/// An ordered pass list for one level.
#[derive(Debug, Clone, Copy)]
pub struct Pipeline {
    /// The level the pipeline runs on.
    pub level: Level,
    /// Passes in order.
    pub passes: &'static [PassDesc],
}

/// Why a pass stopped the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassFailure {
    /// Static reason.
    pub reason: &'static str,
}

/// Runs pipelines and reports to one observer. `NoObserver` compiles away.
#[derive(Debug, Default)]
pub struct PassManager<O: PassObserver> {
    observer: O,
}

impl<O: PassObserver> PassManager<O> {
    /// A manager reporting to `observer`.
    #[must_use]
    pub const fn new(observer: O) -> Self {
        Self { observer }
    }

    /// Run `pipeline`, calling `run_pass` for each pass in fused order.
    pub fn run<F>(&mut self, pipeline: &Pipeline, run_pass: F) -> Result<(), PassFailure>
    where
        F: FnMut(&PassDesc) -> Result<(), PassFailure>,
    {
        let _ = (pipeline, run_pass);
        todo!()
    }

    /// The observer, for reading what it collected.
    pub fn into_observer(self) -> O {
        self.observer
    }
}
