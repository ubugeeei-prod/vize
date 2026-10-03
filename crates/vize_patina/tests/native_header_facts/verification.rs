use super::support::{selected, span};
use vize_l0::{
    Allocator, Span,
    diag::{WitnessChain, WitnessKey, WitnessLink, verify::WitnessError},
    fact::{Demand, FactConsumer, FactError, FactGroup, ids},
    pass::AnalysisId,
};
use vize_patina::native::{
    NativeSyntaxLint,
    header_facts::{
        NativeLintAttributes, NativeLintHeaders, NativeUnsupportedAria, UnsupportedAriaDemand,
    },
};

#[test]
fn real_sdk_checks_causal_header_attribute_and_derived_counterexample_links() {
    let source = "<template><meta title='safe' :role='opaque' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let facts = lint.header_facts(&element).unwrap();
    let chain = facts.unsupported_aria_chain(1).unwrap().unwrap();
    assert_eq!(
        chain.links(),
        [
            WitnessLink::of::<NativeLintHeaders>(&0, span(source, "meta")),
            WitnessLink::of::<NativeLintAttributes>(&1, span(source, ":role='opaque'")),
            WitnessLink::of::<NativeUnsupportedAria>(&1, span(source, ":role='opaque'")),
        ]
    );
    assert_eq!(facts.verify(&chain), Ok(()));
    assert_eq!(facts.unsupported_aria_chain(0).unwrap(), None);
    assert_eq!(facts.verify(&chain), Ok(()));
    assert_eq!(UnsupportedAriaDemand::DEMAND.len(), 3);
    assert_eq!(NativeLintHeaders::STRATUM, 0);
    assert_eq!(NativeLintAttributes::STRATUM, 0);
    assert_eq!(NativeUnsupportedAria::STRATUM, 1);
}

#[test]
fn missing_and_wrong_shape_keys_are_exact_refusals_in_real_sdk_verification() {
    let source = "<template><meta role='presentation' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let facts = lint.header_facts(&element).unwrap();
    let range = span(source, "role='presentation'");
    for group in [
        NativeLintHeaders::ID,
        NativeLintAttributes::ID,
        NativeUnsupportedAria::ID,
    ] {
        let key = WitnessKey::Index(91);
        assert_eq!(
            facts.verify(&WitnessChain::new(WitnessLink::new(
                group,
                range,
                key.clone()
            ))),
            Err(WitnessError::MissingKey {
                link: 0,
                group,
                key
            })
        );
        let key = WitnessKey::Artifact;
        assert_eq!(
            facts.verify(&WitnessChain::new(WitnessLink::new(
                group,
                range,
                key.clone()
            ))),
            Err(WitnessError::KeyShape {
                link: 0,
                group,
                key
            })
        );
    }
    let group = AnalysisId::new(ids::FIXTURE_BASE);
    assert_eq!(
        facts.verify(&WitnessChain::new(WitnessLink::new(
            group,
            range,
            WitnessKey::Index(0)
        ))),
        Err(WitnessError::UnknownGroup { link: 0, group })
    );
}

#[test]
fn each_original_full_range_is_checked_and_later_link_reports_its_exact_index() {
    let source = "<template><meta role='presentation' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let facts = lint.header_facts(&element).unwrap();
    let original = facts.unsupported_aria_chain(0).unwrap().unwrap();
    for index in 0..3 {
        let mut links = original.links().to_vec();
        let fact = links[index].span;
        let witness = Span::new(fact.start + 1, fact.end);
        let group = links[index].group;
        links[index].span = witness;
        assert_eq!(
            facts.verify(&WitnessChain::from_links(links).unwrap()),
            Err(WitnessError::SpanMismatch {
                link: index,
                group,
                fact,
                witness
            })
        );
    }
}

#[test]
fn declared_sdk_views_reject_undeclared_access_and_group_type_aliases() {
    struct EmptyDemand;
    impl FactConsumer for EmptyDemand {
        const NAME: &'static str = "native-header-fixture-empty-demand";
        const DEMAND: Demand = Demand::NONE;
    }
    struct WrongHeaderType;
    impl FactGroup for WrongHeaderType {
        const ID: AnalysisId = ids::NATIVE_LINT_HEADERS;
        const NAME: &'static str = "native-header-fixture-wrong-type";
        const STRATUM: u8 = 0;
        const DEPENDS: Demand = Demand::NONE;
        type Key = u32;
        type Value = Span;
    }
    let source = "<template><meta role='presentation' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let facts = lint.header_facts(&element).unwrap();
    #[cfg(debug_assertions)]
    assert_eq!(
        facts
            .facts::<EmptyDemand>()
            .get::<NativeLintHeaders>()
            .err(),
        Some(FactError::Undeclared {
            consumer: EmptyDemand::NAME,
            group: NativeLintHeaders::ID
        })
    );
    assert_eq!(
        facts
            .facts::<UnsupportedAriaDemand>()
            .get::<WrongHeaderType>()
            .err(),
        Some(FactError::TypeMismatch {
            group: NativeLintHeaders::ID,
            expected: core::any::type_name::<WrongHeaderType>()
        })
    );
}
