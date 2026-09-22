//! The `UndefinedRefs` fact group (P4-3a): template identifiers that no
//! binding, template scope or builtin resolves.
//!
//! Keyed by walk-order ordinal: the relation is a sequence — the drawer
//! reports each unresolved read in template order, a repeated read of the
//! same name is a repeated fact, and consumers report in that order. The
//! TS-34 spec compares the relation as a multiset of `(name, offset,
//! context)` facts ([`crate::facts::spec::undefined_refs`]).

use vize_davinci::fact::{Demand, FactGroup, FactProducer, FactTable, FactView, ids};
use vize_davinci::pass::AnalysisId;

use crate::{Croquis, UndefinedRef};

/// The `UndefinedRefs` fact group.
pub struct UndefinedRefs;

impl FactGroup for UndefinedRefs {
    const ID: AnalysisId = ids::UNDEFINED_REFS;
    const NAME: &'static str = "undefined-refs";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = UndefinedRef;
}

impl FactProducer<Croquis> for UndefinedRefs {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        (0u32..)
            .zip(croquis.undefined_refs.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::UndefinedRefs;
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use crate::{Croquis, UndefinedRef};
    use vize_carton::CompactString;

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/undefined-refs-reader";
        const DEMAND: Demand = Demand::NONE.with(UndefinedRefs::ID);
    }

    fn reference(name: &str, offset: u32) -> UndefinedRef {
        UndefinedRef {
            name: CompactString::new(name),
            offset,
            context: CompactString::new("template expression"),
        }
    }

    #[test]
    fn walk_order_and_repeats_survive() {
        let mut croquis = Croquis::new();
        croquis.undefined_refs = vec![reference("b", 30), reference("a", 10), reference("b", 30)];
        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<UndefinedRefs>().unwrap();
        let entries: Vec<(u32, UndefinedRef)> = table
            .iter()
            .map(|(ordinal, fact)| (*ordinal, fact.clone()))
            .collect();
        assert_eq!(
            entries,
            vec![
                (0, reference("b", 30)),
                (1, reference("a", 10)),
                (2, reference("b", 30))
            ]
        );
    }
}
