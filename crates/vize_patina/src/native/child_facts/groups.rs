use super::{Evidence, NativeInterpolationFact, NativeLintHeaders, NativeTextareaMustacheFact};
use vize_l0::{
    Span,
    diag::verify::{WitnessCheck, WitnessChecks, WitnessGroup},
    fact::{
        Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView,
        ProducerEntry, ids,
    },
    pass::AnalysisId,
};

// Copy only the already authenticated header obtained through its declared SDK
// view. The existing header registry and producer remain completely unchanged.
impl FactProducer<Evidence> for NativeLintHeaders {
    fn produce(evidence: &Evidence, _: &FactView<'_>) -> FactTable<Self> {
        [(evidence.header_key, evidence.header.clone())]
            .into_iter()
            .collect()
    }
}
pub(super) struct HeaderDemand;
impl FactConsumer for HeaderDemand {
    const NAME: &'static str = "native-direct-child-header";
    const DEMAND: Demand = Demand::NONE.with(ids::NATIVE_LINT_HEADERS);
}

pub struct NativeDirectInterpolations;
impl FactGroup for NativeDirectInterpolations {
    const ID: AnalysisId = ids::NATIVE_DIRECT_INTERPOLATIONS;
    const NAME: &'static str = "native-direct-interpolations";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = NativeInterpolationFact;
}
impl FactProducer<Evidence> for NativeDirectInterpolations {
    fn produce(evidence: &Evidence, _: &FactView<'_>) -> FactTable<Self> {
        evidence.marker.into_iter().collect()
    }
}
impl WitnessGroup for NativeDirectInterpolations {
    fn fact_span(_: &u32, marker: &NativeInterpolationFact) -> Span {
        marker.span
    }
}

pub struct NativeTextareaMustache;
impl FactGroup for NativeTextareaMustache {
    const ID: AnalysisId = ids::NATIVE_TEXTAREA_MUSTACHE;
    const NAME: &'static str = "native-textarea-mustache";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE
        .with(ids::NATIVE_LINT_HEADERS)
        .with(ids::NATIVE_DIRECT_INTERPOLATIONS);
    type Key = u32;
    type Value = NativeTextareaMustacheFact;
}
impl FactProducer<Evidence> for NativeTextareaMustache {
    fn produce(_: &Evidence, inputs: &FactView<'_>) -> FactTable<Self> {
        let (Ok(headers), Ok(markers)) = (
            inputs.get::<NativeLintHeaders>(),
            inputs.get::<NativeDirectInterpolations>(),
        ) else {
            return FactTable::default();
        };
        let Some((header_key, header)) = headers.iter().next() else {
            return FactTable::default();
        };
        if header.tag() != "textarea" {
            return FactTable::default();
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

pub struct TextareaMustacheDemand;
impl FactConsumer for TextareaMustacheDemand {
    const NAME: &'static str = "vue/no-textarea-mustache";
    const DEMAND: Demand = Demand::NONE
        .with(ids::NATIVE_LINT_HEADERS)
        .with(ids::NATIVE_DIRECT_INTERPOLATIONS)
        .with(ids::NATIVE_TEXTAREA_MUSTACHE);
}
pub(super) static REGISTRY: FactRegistry<Evidence> = FactRegistry::new(&[
    ProducerEntry::of::<NativeLintHeaders>(),
    ProducerEntry::of::<NativeDirectInterpolations>(),
    ProducerEntry::of::<NativeTextareaMustache>(),
]);
pub(super) static CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<NativeLintHeaders>(),
    WitnessCheck::of::<NativeDirectInterpolations>(),
    WitnessCheck::of::<NativeTextareaMustache>(),
]);
