use super::support::{selected, span};
use vize_l0::{
    Allocator, Span,
    diag::{WitnessChain, WitnessKey, WitnessLink, verify::WitnessError},
    fact::{Demand, FactConsumer, FactError, FactGroup, ids},
    pass::AnalysisId,
};
use vize_patina::native::{
    NativeSyntaxLint,
    child_facts::{NativeDirectInterpolations, NativeTextareaMustache, TextareaMustacheDemand},
    header_facts::NativeLintHeaders,
};

#[test]
fn real_sdk_checks_original_parent_marker_and_derived_counterexample_links() {
    let source = "<template><textarea>safe{{opaque}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let ordinal = 1;
    let facts = header
        .child_facts(element.children().nth(ordinal).unwrap())
        .unwrap();
    let chain = facts.textarea_mustache_chain(1).unwrap().unwrap();
    assert_eq!(
        chain.links(),
        [
            WitnessLink::of::<NativeLintHeaders>(&0, span(source, "textarea")),
            WitnessLink::of::<NativeDirectInterpolations>(&1, span(source, "{{opaque}}")),
            WitnessLink::of::<NativeTextareaMustache>(&1, span(source, "{{opaque}}")),
        ]
    );
    assert_eq!(facts.verify(&chain), Ok(()));
    assert_eq!(facts.textarea_mustache_chain(0).unwrap(), None);
    assert_eq!(facts.verify(&chain), Ok(()));
    assert_eq!(TextareaMustacheDemand::DEMAND.len(), 3);
    assert_eq!(NativeLintHeaders::STRATUM, 0);
    assert_eq!(NativeDirectInterpolations::STRATUM, 0);
    assert_eq!(NativeTextareaMustache::STRATUM, 1);
}

#[test]
fn missing_and_wrong_shape_keys_are_exact_refusals_in_real_sdk_verification() {
    let source = "<template><textarea>{{presentation}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let ordinal = 0;
    let facts = header
        .child_facts(element.children().nth(ordinal).unwrap())
        .unwrap();
    let range = span(source, "{{presentation}}");
    for group in [
        NativeLintHeaders::ID,
        NativeDirectInterpolations::ID,
        NativeTextareaMustache::ID,
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
    let source = "<template><textarea>{{presentation}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let ordinal = 0;
    let facts = header
        .child_facts(element.children().nth(ordinal).unwrap())
        .unwrap();
    let original = facts.textarea_mustache_chain(0).unwrap().unwrap();
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
        const NAME: &'static str = "native-child-fixture-empty-demand";
        const DEMAND: Demand = Demand::NONE;
    }
    struct WrongHeaderType;
    impl FactGroup for WrongHeaderType {
        const ID: AnalysisId = ids::NATIVE_LINT_HEADERS;
        const NAME: &'static str = "native-child-fixture-wrong-type";
        const STRATUM: u8 = 0;
        const DEPENDS: Demand = Demand::NONE;
        type Key = u32;
        type Value = Span;
    }
    let source = "<template><textarea>{{presentation}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let ordinal = 0;
    let facts = header
        .child_facts(element.children().nth(ordinal).unwrap())
        .unwrap();
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
            .facts::<TextareaMustacheDemand>()
            .get::<WrongHeaderType>()
            .err(),
        Some(FactError::TypeMismatch {
            group: NativeLintHeaders::ID,
            expected: core::any::type_name::<WrongHeaderType>()
        })
    );
}
