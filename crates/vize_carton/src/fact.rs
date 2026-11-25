//! Carton's historical fact paths, retained for one published minor release.
//!
//! Canonical analysis APIs live in [`crate::analysis`]. Re-exports retain trait,
//! enum and unit-constructor identity; concrete aliases retain method and field
//! usage. See the level-vocabulary compatibility decision for the removal window.

#![expect(
    deprecated,
    reason = "the compatibility module re-exports its deprecated historical names"
)]

pub use vize_l0::analysis::{
    ALPHA_GROUPS, AlphaDesc, AlphaDocError, AlphaDocument, AlphaExport, Demand, GroupDesc,
    PRESERVE_BINDINGS, PRESERVE_STRUCTURE, ProducerEntry, StrataError, check_alpha_schema_doc,
    check_strata, produced_count, undeclared_accesses,
};
pub use vize_l0::analysis::{alpha, demand, ids};

/// Use [`vize_l0::analysis::Analysis`].
#[deprecated(note = "use vize_carton::analysis::Analysis")]
pub use vize_l0::analysis::Analysis as FactGroup;
/// Use [`vize_l0::analysis::AnalysisConsumer`].
#[deprecated(note = "use vize_carton::analysis::AnalysisConsumer")]
pub use vize_l0::analysis::AnalysisConsumer as FactConsumer;
/// Use [`vize_l0::analysis::AnalysisProducer`].
#[deprecated(note = "use vize_carton::analysis::AnalysisProducer")]
pub use vize_l0::analysis::AnalysisProducer as FactProducer;

pub use expression::{ExpressionFact, ExpressionFacts, ExpressionFactsAlpha};
pub use manager::FactManager;
pub use preserve::{FactVerify, FactVerifyObserver, NoFactVerify};
pub use registry::FactRegistry;
pub use table::{FactTable, FactTableBuilder};
pub use view::{FactError, FactView};

/// Historical expression-analysis paths.
#[deprecated(note = "use vize_carton::analysis::expression")]
pub mod expression {
    /// Use [`vize_l0::analysis::ExpressionAnalysis`].
    #[deprecated(note = "use vize_carton::analysis::ExpressionAnalysis")]
    pub use vize_l0::analysis::ExpressionAnalysis as ExpressionFacts;
    /// Use [`vize_l0::analysis::ExpressionInfo`].
    #[deprecated(note = "use vize_carton::analysis::ExpressionInfo")]
    pub type ExpressionFact = vize_l0::analysis::ExpressionInfo;
    /// Use [`vize_l0::analysis::ExpressionAnalysisAlpha`].
    #[deprecated(note = "use vize_carton::analysis::ExpressionAnalysisAlpha")]
    pub type ExpressionFactsAlpha = vize_l0::analysis::ExpressionAnalysisAlpha;
}

/// Historical analysis-manager paths.
#[deprecated(note = "use vize_carton::analysis::manager")]
pub mod manager {
    /// Use [`vize_l0::analysis::AnalysisManager`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisManager")]
    pub type FactManager<'r, A> = vize_l0::analysis::AnalysisManager<'r, A>;
}

/// Historical analysis-preservation paths.
#[deprecated(note = "use vize_carton::analysis::preserve")]
pub mod preserve {
    /// Use [`vize_l0::analysis::AnalysisVerify`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisVerify")]
    pub use vize_l0::analysis::AnalysisVerify as FactVerify;
    /// Use [`vize_l0::analysis::AnalysisVerifyObserver`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisVerifyObserver")]
    pub use vize_l0::analysis::AnalysisVerifyObserver as FactVerifyObserver;
    /// Use [`vize_l0::analysis::NoAnalysisVerify`].
    #[deprecated(note = "use vize_carton::analysis::NoAnalysisVerify")]
    pub use vize_l0::analysis::NoAnalysisVerify as NoFactVerify;
    pub use vize_l0::analysis::preserve::{PRESERVE_BINDINGS, PRESERVE_STRUCTURE};
}

/// Historical analysis-registry paths.
#[deprecated(note = "use vize_carton::analysis::registry")]
pub mod registry {
    pub use vize_l0::analysis::registry::{GroupDesc, ProducerEntry, StrataError, check_strata};
    /// Use [`vize_l0::analysis::AnalysisRegistry`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisRegistry")]
    pub type FactRegistry<A> = vize_l0::analysis::AnalysisRegistry<A>;
}

/// Historical analysis-table paths.
#[deprecated(note = "use vize_carton::analysis::table")]
pub mod table {
    /// Use [`vize_l0::analysis::AnalysisTable`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisTable")]
    pub type FactTable<G> = vize_l0::analysis::AnalysisTable<G>;
    /// Use [`vize_l0::analysis::AnalysisTableBuilder`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisTableBuilder")]
    pub type FactTableBuilder<G> = vize_l0::analysis::AnalysisTableBuilder<G>;
}

/// Historical analysis-query paths.
#[deprecated(note = "use vize_carton::analysis::view")]
pub mod view {
    /// Use [`vize_l0::analysis::AnalysisError`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisError")]
    pub use vize_l0::analysis::AnalysisError as FactError;
    pub use vize_l0::analysis::view::{produced_count, undeclared_accesses};
    /// Use [`vize_l0::analysis::AnalysisView`].
    #[deprecated(note = "use vize_carton::analysis::AnalysisView")]
    pub type FactView<'m> = vize_l0::analysis::AnalysisView<'m>;
}
