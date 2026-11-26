use super::{Evidence, NativeInterpolationFact, NativeLintHeaders, NativeTextareaMustacheFact};
use vize_l0::{
    Span,
    analysis::{
        Analysis, AnalysisConsumer, AnalysisProducer, AnalysisRegistry, AnalysisTable,
        AnalysisView, Demand, ProducerEntry, ids,
    },
    diag::verify::{WitnessCheck, WitnessChecks, WitnessGroup},
    pass::AnalysisId,
};

// Copy only the already authenticated header obtained through its declared SDK
// view. The existing header registry and producer remain completely unchanged.
impl AnalysisProducer<Evidence> for NativeLintHeaders {
    fn produce(evidence: &Evidence, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        [(evidence.header_key, evidence.header.clone())]
            .into_iter()
            .collect()
    }
}
pub(super) struct HeaderDemand;
impl AnalysisConsumer for HeaderDemand {
    const NAME: &'static str = "native-direct-child-header";
    const DEMAND: Demand = Demand::NONE.with(ids::ORIGINAL_LINT_HEADERS);
}

/// Fact group containing closed interpolation markers from one direct child.
pub struct NativeDirectInterpolations;
impl Analysis for NativeDirectInterpolations {
    const ID: AnalysisId = ids::ORIGINAL_DIRECT_INTERPOLATIONS;
    const NAME: &'static str = "native-direct-interpolations";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = NativeInterpolationFact;
}
impl AnalysisProducer<Evidence> for NativeDirectInterpolations {
    fn produce(evidence: &Evidence, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        evidence.marker.into_iter().collect()
    }
}
impl WitnessGroup for NativeDirectInterpolations {
    fn fact_span(_: &u32, marker: &NativeInterpolationFact) -> Span {
        marker.span
    }
}

/// Fact group containing textarea interpolation counterexamples.
pub struct NativeTextareaMustache;
impl Analysis for NativeTextareaMustache {
    const ID: AnalysisId = ids::ORIGINAL_TEXTAREA_MUSTACHE;
    const NAME: &'static str = "native-textarea-mustache";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE
        .with(ids::ORIGINAL_LINT_HEADERS)
        .with(ids::ORIGINAL_DIRECT_INTERPOLATIONS);
    type Key = u32;
    type Value = NativeTextareaMustacheFact;
}
impl AnalysisProducer<Evidence> for NativeTextareaMustache {
    fn produce(_: &Evidence, inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let (Ok(headers), Ok(markers)) = (
            inputs.get::<NativeLintHeaders>(),
            inputs.get::<NativeDirectInterpolations>(),
        ) else {
            return AnalysisTable::default();
        };
        let Some((header_key, header)) = headers.iter().next() else {
            return AnalysisTable::default();
        };
        if header.tag() != "textarea" {
            return AnalysisTable::default();
        }
        markers
            .iter()
            .filter(|(_, marker)| marker.header == *header_key)
            .map(|(ordinal, marker)| {
                (
                    *ordinal,
                    NativeTextareaMustacheFact {
                        header: *header_key,
                        span: marker.span,
                    },
                )
            })
            .collect()
    }
}
impl WitnessGroup for NativeTextareaMustache {
    fn fact_span(_: &u32, fact: &NativeTextareaMustacheFact) -> Span {
        fact.span
    }
}

/// Declared fact demand for the `vue/no-textarea-mustache` native query.
pub struct TextareaMustacheDemand;
impl AnalysisConsumer for TextareaMustacheDemand {
    const NAME: &'static str = "vue/no-textarea-mustache";
    const DEMAND: Demand = Demand::NONE
        .with(ids::ORIGINAL_LINT_HEADERS)
        .with(ids::ORIGINAL_DIRECT_INTERPOLATIONS)
        .with(ids::ORIGINAL_TEXTAREA_MUSTACHE);
}
pub(super) static REGISTRY: AnalysisRegistry<Evidence> = AnalysisRegistry::new(&[
    ProducerEntry::of::<NativeLintHeaders>(),
    ProducerEntry::of::<NativeDirectInterpolations>(),
    ProducerEntry::of::<NativeTextareaMustache>(),
]);
pub(super) static CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<NativeLintHeaders>(),
    WitnessCheck::of::<NativeDirectInterpolations>(),
    WitnessCheck::of::<NativeTextareaMustache>(),
]);
