//! External-consumer coverage for the compatibility-preview analysis facade.

#![expect(
    deprecated,
    reason = "the supported old names remain usable for one minor release"
)]
use vize_croquis::facts::FactError::NotComputed;
use vize_croquis::facts::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactRegistry, FactTable, FactView,
    ProducerEntry,
};
use vize_l0::analysis::AnalysisProducer;
use vize_l0::pass::AnalysisId;

struct Values;
impl FactGroup for Values {
    const ID: AnalysisId = AnalysisId::new(32);
    const NAME: &'static str = "compat-values";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = u32;
}
impl AnalysisProducer<[u32]> for Values {
    fn produce(numbers: &[u32], _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(numbers.iter().copied()).collect()
    }
}
struct Reader;
impl FactConsumer for Reader {
    const NAME: &'static str = "compat-reader";
    const DEMAND: Demand = Demand::NONE.with(Values::ID);
}
const REGISTRY: FactRegistry<[u32]> = FactRegistry::new(&[ProducerEntry::of::<Values>()]);

#[test]
fn the_deprecated_facade_still_supports_an_external_unsized_consumer() {
    let mut manager: FactManager<'_, [u32]> = FactManager::new(&REGISTRY);
    assert_eq!(
        manager.view::<Reader>().get::<Values>().err(),
        Some(NotComputed { group: Values::ID }),
    );
    let view: FactView<'_> = manager.prepare::<Reader>(&[3, 5]).expect("declared demand");
    let values: Result<&FactTable<Values>, FactError> = view.get::<Values>();
    assert_eq!(
        values.expect("the old table name remains usable").get(&1),
        Some(&5)
    );
}
