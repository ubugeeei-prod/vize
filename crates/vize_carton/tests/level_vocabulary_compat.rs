//! External-consumer coverage for Carton's supported historical fact subtree.

#![expect(
    deprecated,
    reason = "the historical module and concrete aliases retain their deprecation window"
)]
use vize_carton::fact;
use vize_carton::fact::FactError::*;
use vize_carton::fact::alpha::AlphaExport;
use vize_carton::fact::demand::Demand as NestedDemand;
use vize_carton::fact::expression::{
    ExpressionFact as NestedExpressionFact, ExpressionFacts as NestedExpressionFacts,
    ExpressionFactsAlpha as NestedExpressionFactsAlpha,
};
use vize_carton::fact::manager::FactManager as NestedManager;
use vize_carton::fact::preserve::{
    FactVerify as NestedVerify, FactVerifyObserver as NestedObserver,
    NoFactVerify as NestedNoVerify,
};
use vize_carton::fact::registry::FactRegistry as NestedRegistry;
use vize_carton::fact::table::{FactTable as NestedTable, FactTableBuilder as NestedBuilder};
use vize_carton::fact::view::{FactError as NestedError, FactView as NestedView};
use vize_carton::fact::{
    Demand, ExpressionFact, ExpressionFacts, ExpressionFactsAlpha, FactConsumer, FactError,
    FactGroup, FactManager, FactProducer, FactRegistry, FactTable, FactTableBuilder, FactVerify,
    FactVerifyObserver, FactView, NoFactVerify, ProducerEntry,
};
use vize_carton::pass::AnalysisId;

struct Values;
impl FactGroup for Values {
    const ID: AnalysisId = AnalysisId::new(32);
    const NAME: &'static str = "carton-compat-values";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = u32;
}
impl FactProducer<[u32]> for Values {
    fn produce(numbers: &[u32], _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(numbers.iter().copied()).collect()
    }
}
struct Reader;
impl FactConsumer for Reader {
    const NAME: &'static str = "carton-compat-reader";
    const DEMAND: Demand = Demand::NONE.with(Values::ID);
}
const REGISTRY: FactRegistry<[u32]> = NestedRegistry::new(&[ProducerEntry::of::<Values>()]);

#[test]
fn old_root_and_nested_names_support_an_external_unsized_producer_and_query() {
    let _: &NestedRegistry<[u32]> = &REGISTRY;
    let _: NestedDemand = Reader::DEMAND;
    let mut manager: FactManager<'_, [u32]> = NestedManager::new(&REGISTRY);
    let missing: FactError = NotComputed { group: Values::ID };
    let _: &NestedError = &missing;
    assert_eq!(
        manager.view::<Reader>().get::<Values>().err(),
        Some(missing)
    );
    let view: FactView<'_> = manager.prepare::<Reader>(&[3, 5]).expect("declared demand");
    let _: &NestedView<'_> = &view;
    let values: &FactTable<Values> = view.get::<Values>().expect("the old producer was computed");
    let _: &NestedTable<Values> = values;
    assert_eq!(values.get(&1), Some(&5));

    let mut builder: NestedBuilder<Values> = FactTableBuilder::with_capacity(1);
    builder.insert(0, 9);
    assert_eq!(builder.finish().get(&0), Some(&9));
}

fn nested_verify<V: NestedVerify>(_: V) {}
fn root_verify<V: FactVerify>(value: V) {
    nested_verify(value);
}

#[test]
fn old_unit_patterns_and_named_field_records_retain_canonical_identity() {
    let ExpressionFacts = NestedExpressionFacts;
    let FactVerifyObserver = NestedObserver;
    let NoFactVerify = NestedNoVerify;
    root_verify(FactVerifyObserver);
    root_verify(NoFactVerify);

    let record: NestedExpressionFact = ExpressionFact {
        references: vize_carton::String::default(),
        exact: true,
        constant: false,
    };
    assert!(record.exact);
    let alpha: ExpressionFactsAlpha = NestedExpressionFactsAlpha::default();
    let _: <ExpressionFacts as AlphaExport>::Alpha = alpha;
    assert_eq!(ExpressionFacts::ID, fact::ids::EXPRESSION_FACTS);
}
