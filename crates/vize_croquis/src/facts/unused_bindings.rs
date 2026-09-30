//! `UnusedBindings`: the authoritative unread setup relation, with script spans.
//!
//! Population is an explicit Drawer demand. The producer only projects the
//! stored relation; it never parses script text or guesses usage.

use vize_l0::fact::{Demand, FactGroup, FactProducer, FactTable, FactView, ids};
use vize_l0::pass::AnalysisId;

use super::{Bindings, BindingsTable};
use crate::Croquis;

/// Definition range in the same script frame as `Bindings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnusedBindingFact {
    /// Exclusive byte range of the declaration identifier.
    pub span: (u32, u32),
}

/// Unread `<script setup>` bindings, keyed by name.
pub struct UnusedBindings;

impl FactGroup for UnusedBindings {
    const ID: AnalysisId = ids::UNUSED_BINDINGS;
    const NAME: &'static str = "unused-bindings";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(Bindings::ID);
    #[expect(
        clippy::disallowed_types,
        reason = "typed name witnesses round trip through the Davinci String key"
    )]
    type Key = String;
    type Value = UnusedBindingFact;
}

impl FactProducer<Croquis> for UnusedBindings {
    fn produce(croquis: &Croquis, view: &FactView<'_>) -> FactTable<Self> {
        let Ok(bindings) = view.get::<Bindings>() else {
            return FactTable::default();
        };
        if !bindings.is_script_setup() {
            return FactTable::default();
        }
        croquis
            .unused_bindings
            .iter()
            .filter_map(|name| {
                bindings
                    .span(name)
                    .map(|span| (name.as_str().into(), UnusedBindingFact { span }))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{CroquisFacts, FactConsumer};

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/unused-bindings";
        const DEMAND: Demand = Demand::NONE.with(UnusedBindings::ID);
    }

    #[test]
    fn authoritative_names_and_binding_spans_are_projected() {
        let mut croquis = Croquis::new();
        croquis.bindings.is_script_setup = true;
        croquis.unused_bindings = vec!["second".into(), "first".into(), "missing".into()];
        croquis.binding_spans.insert("first".into(), (6, 11));
        croquis.binding_spans.insert("second".into(), (30, 36));
        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<UnusedBindings>().unwrap();
        assert_eq!(
            table
                .iter()
                .map(|(name, fact)| (name.as_str(), fact.span))
                .collect::<Vec<_>>(),
            [("first", (6, 11)), ("second", (30, 36))]
        );
        let again = facts.prepare::<Reader>().get::<UnusedBindings>().unwrap();
        assert_eq!(again.len(), 2);
    }
}
