//! One compiled copy of each provenance-generic pass per sink.
//!
//! The pass bodies are generic over the provenance sink, and a generic body
//! is compiled again in every crate that instantiates it. The transform
//! drivers are generic over their observer, so every consumer crate used to
//! compile its own copy of every pass. The drivers reach the passes through
//! [`PassSink`] instead: its impls are not generic, so each pass is compiled
//! once per sink, in this crate.

use vize_l0::side_table::SideTable;
use vize_l2::analysis::pass_facts::FilterAssets;
use vize_l2::provenance::{NoProvenance, ProvenanceRecord, ProvenanceSink};

use super::{ModelFacts, SlotFacts, StaticFacts, hoist, legacy, vmodel, vslot};
use crate::lower::Lowered;

mod sealed {
    pub trait Sealed {}
    impl Sealed for alloc::vec::Vec<vize_l2::provenance::ProvenanceRecord> {}
    impl Sealed for vize_l2::provenance::NoProvenance {}
}

/// A provenance sink the transform drivers run their passes over: the
/// recording vector or [`NoProvenance`].
pub trait PassSink: ProvenanceSink + Sized + sealed::Sealed {
    #[doc(hidden)]
    fn legacy(lowered: &mut Lowered<'_, Self>) -> FilterAssets;
    #[doc(hidden)]
    fn slots(lowered: &mut Lowered<'_, Self>) -> SideTable<SlotFacts>;
    #[doc(hidden)]
    fn models(lowered: &mut Lowered<'_, Self>) -> SideTable<ModelFacts>;
    #[doc(hidden)]
    fn statics(lowered: &mut Lowered<'_, Self>) -> SideTable<StaticFacts>;
}

macro_rules! pass_sink {
    ($sink:ty) => {
        impl PassSink for $sink {
            fn legacy(lowered: &mut Lowered<'_, Self>) -> FilterAssets {
                legacy::run(lowered)
            }

            fn slots(lowered: &mut Lowered<'_, Self>) -> SideTable<SlotFacts> {
                vslot::run(lowered)
            }

            fn models(lowered: &mut Lowered<'_, Self>) -> SideTable<ModelFacts> {
                vmodel::run(lowered)
            }

            fn statics(lowered: &mut Lowered<'_, Self>) -> SideTable<StaticFacts> {
                hoist::run(lowered)
            }
        }
    };
}

pass_sink!(alloc::vec::Vec<ProvenanceRecord>);
pass_sink!(NoProvenance);
