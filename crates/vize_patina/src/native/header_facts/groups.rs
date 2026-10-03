use vize_l0::{
    Span,
    diag::verify::{WitnessCheck, WitnessChecks, WitnessGroup},
    fact::{
        Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView,
        ProducerEntry, ids,
    },
    pass::AnalysisId,
};
use vize_l1::markup::NativeLintTagKind;

use super::{
    Evidence, NativeAttributeFact, NativeBindingKind, NativeHeaderFact, NativeUnsupportedAriaFact,
};

/// Original authored header fact group, scoped to one genuine element.
pub struct NativeLintHeaders;
impl FactGroup for NativeLintHeaders {
    const ID: AnalysisId = ids::NATIVE_LINT_HEADERS;
    const NAME: &'static str = "native-lint-headers";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = NativeHeaderFact;
}
impl FactProducer<Evidence> for NativeLintHeaders {
    fn produce(evidence: &Evidence, _: &FactView<'_>) -> FactTable<Self> {
        [(evidence.header_key, evidence.header.clone())]
            .into_iter()
            .collect()
    }
}
impl WitnessGroup for NativeLintHeaders {
    fn fact_span(_: &u32, header: &NativeHeaderFact) -> Span {
        header.span
    }
}

/// Checked original authored attributes, with their real attribute ordinals.
pub struct NativeLintAttributes;
impl FactGroup for NativeLintAttributes {
    const ID: AnalysisId = ids::NATIVE_LINT_ATTRIBUTES;
    const NAME: &'static str = "native-lint-attributes";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = NativeAttributeFact;
}
impl FactProducer<Evidence> for NativeLintAttributes {
    fn produce(evidence: &Evidence, _: &FactView<'_>) -> FactTable<Self> {
        evidence.attributes.iter().cloned().collect()
    }
}
impl WitnessGroup for NativeLintAttributes {
    fn fact_span(_: &u32, attribute: &NativeAttributeFact) -> Span {
        attribute.span
    }
}

/// Authored rule counterexamples, derived solely from genuine header and attrs.
pub struct NativeUnsupportedAria;
impl FactGroup for NativeUnsupportedAria {
    const ID: AnalysisId = ids::NATIVE_UNSUPPORTED_ARIA;
    const NAME: &'static str = "native-unsupported-aria";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE
        .with(ids::NATIVE_LINT_HEADERS)
        .with(ids::NATIVE_LINT_ATTRIBUTES);
    type Key = u32;
    type Value = NativeUnsupportedAriaFact;
}
impl FactProducer<Evidence> for NativeUnsupportedAria {
    fn produce(_: &Evidence, inputs: &FactView<'_>) -> FactTable<Self> {
        let (Ok(headers), Ok(attributes)) = (
            inputs.get::<NativeLintHeaders>(),
            inputs.get::<NativeLintAttributes>(),
        ) else {
            return FactTable::default();
        };
        let Some((header_key, header)) = headers.iter().next() else {
            return FactTable::default();
        };
        // This is the exact registered lint rule's authored-name contract, not
        // a DOM/runtime classification or a legacy semantic helper invocation.
        if header.kind == NativeLintTagKind::Component
            || !matches!(header.tag.as_str(), "meta" | "html" | "script" | "style")
        {
            return FactTable::default();
        }
        attributes
            .iter()
            .filter(|(_, attr)| {
                attr.kind != NativeBindingKind::Other
                    && (attr.name.starts_with("aria-") || attr.name.as_str() == "role")
            })
            .map(|(key, attr)| {
                (
                    *key,
                    NativeUnsupportedAriaFact {
                        header: *header_key,
                        span: attr.span,
                    },
                )
            })
            .collect()
    }
}
impl WitnessGroup for NativeUnsupportedAria {
    fn fact_span(_: &u32, fact: &NativeUnsupportedAriaFact) -> Span {
        fact.span
    }
}

/// The complete demand for the future witnessed rule consumer.
pub struct UnsupportedAriaDemand;
impl FactConsumer for UnsupportedAriaDemand {
    const NAME: &'static str = "a11y/aria-unsupported-elements";
    const DEMAND: Demand = Demand::NONE
        .with(ids::NATIVE_LINT_HEADERS)
        .with(ids::NATIVE_LINT_ATTRIBUTES)
        .with(ids::NATIVE_UNSUPPORTED_ARIA);
}

pub(super) static REGISTRY: FactRegistry<Evidence> = FactRegistry::new(&[
    ProducerEntry::of::<NativeLintHeaders>(),
    ProducerEntry::of::<NativeLintAttributes>(),
    ProducerEntry::of::<NativeUnsupportedAria>(),
]);
pub(super) static CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<NativeLintHeaders>(),
    WitnessCheck::of::<NativeLintAttributes>(),
    WitnessCheck::of::<NativeUnsupportedAria>(),
]);
