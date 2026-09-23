//! The `RaceConditions` fact group (P4-3f).
//!
//! Parser-derived race risks in tracker order.

use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactTable, FactTableBuilder, FactView, ids,
};
use vize_davinci::pass::AnalysisId;

use crate::Croquis;
use crate::race::RaceConditionRisk;

/// The `RaceConditions` fact group.
pub struct RaceConditions;

impl FactGroup for RaceConditions {
    const ID: AnalysisId = ids::RACE_CONDITIONS;
    const NAME: &'static str = "race-conditions";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = RaceConditionRisk;
}

impl FactProducer<Croquis> for RaceConditions {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let mut builder =
            FactTableBuilder::<Self>::with_capacity(croquis.race_conditions.risks().len());
        for (ordinal, risk) in croquis.race_conditions.risks().iter().enumerate() {
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            builder.insert(ordinal, risk.clone());
        }
        builder.finish()
    }
}

struct RaceReader;

impl FactConsumer for RaceReader {
    const NAME: &'static str = "croquis/race-conditions";
    const DEMAND: Demand = Demand::NONE.with(RaceConditions::ID);
}

/// Race risks in tracker order.
#[must_use]
pub fn race_risks(croquis: &Croquis) -> Vec<RaceConditionRisk> {
    let mut facts = super::CroquisFacts::new(croquis);
    let table = facts
        .prepare::<RaceReader>()
        .get::<RaceConditions>()
        .expect("race-conditions demand");
    table.iter().map(|(_, risk)| risk.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::{RaceConditions, race_risks};
    use crate::Croquis;
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use crate::race::RaceConditionRiskKind;
    use vize_carton::CompactString;

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/race-reader";
        const DEMAND: Demand = Demand::NONE.with(RaceConditions::ID);
    }

    #[test]
    fn race_rows_follow_the_tracker() {
        let mut croquis = Croquis::new();
        croquis.race_conditions.record(
            RaceConditionRiskKind::AsyncWatchEffect {
                async_operation: CompactString::new("await fetch()"),
                mutated_targets: vec![CompactString::new("state")],
            },
            8,
            40,
        );
        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<RaceConditions>().unwrap();
        assert_eq!(table.len(), 1);
        assert_eq!(race_risks(&croquis), croquis.race_conditions.risks());
    }
}
