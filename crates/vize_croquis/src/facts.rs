//! Croquis products as fact groups (Davinci P4-3).
//!
//! Each tracker product a consumer reads becomes a fact group on the
//! [`vize_davinci::fact`] query surface: the tracker code stays the
//! population pass (semantic-engine.md #1), a registered producer turns its
//! output into a key-sorted [`FactTable`], and a consumer reads the table
//! through a [`FactView`] scoped to the demand it declared as const data
//! ([`FactConsumer::DEMAND`]). The debug detector refuses every read outside
//! that demand (TS-35), so "consumed but undeclared" is caught mechanically.
//!
//! The artifact the producers read is the drawn [`Croquis`]: its fields stay
//! the producers' storage until lane C deletes the projection generators
//! (P4-5c), the last struct-field readers.
//!
//! # Groups
//!
//! | group | key | wave |
//! | ----- | --- | ---- |
//! | [`Bindings`] | [`BindingKey`] — the script-setup marker, then binding names | P4-3a |
//! | [`UndefinedRefs`] | walk-order ordinal | P4-3a |
//! | [`ComponentUsages`] | [`ComponentIdentity`] — module plus exported name | P4-3b |
//! | [`Reactivity`] | registration ordinal, then loss ordinal | P4-3d |
//!
//! Every group carries a declarative specification and a naive evaluator in
//! [`spec`] (TS-34, the Polonius discipline).
//!
//! # Reading a group
//!
//! ```
//! use vize_croquis::Croquis;
//! use vize_croquis::facts::{
//!     BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup, bindings::Bindings,
//! };
//! use vize_relief::BindingType;
//!
//! struct CountRefs;
//! impl FactConsumer for CountRefs {
//!     const NAME: &'static str = "doc/count-refs";
//!     const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
//! }
//!
//! let mut croquis = Croquis::new();
//! croquis.bindings.add("count", BindingType::SetupRef);
//! let mut facts = CroquisFacts::new(&croquis);
//! let bindings = facts.prepare::<CountRefs>().get::<Bindings>().unwrap();
//! assert_eq!(bindings.binding_type("count"), Some(BindingType::SetupRef));
//! assert!(!bindings.is_script_setup());
//! ```

pub mod bindings;
pub mod components;
pub mod reactivity;
pub mod spec;
pub mod undefined_refs;

pub use bindings::{BindingFact, BindingKey, Bindings, BindingsTable};
pub use components::{ComponentIdentity, ComponentUsages, GroupedComponentUse, component_identity};
pub use reactivity::{
    Reactivity, ReactivityFact, ReactivityKey, SourceFact, reactivity_count, reactivity_has_losses,
    reactivity_is_reactive, reactivity_lookup, reactivity_losses, reactivity_sources,
};

mod access;
pub use access::{
    component_usage_list, used_component_contains, used_component_name_list, used_components_empty,
};
pub use undefined_refs::UndefinedRefs;
pub use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactRegistry, FactTable, FactView,
    ProducerEntry,
};

use crate::Croquis;

/// Every Croquis fact group, stratified at compile time.
pub const CROQUIS_FACTS: FactRegistry<Croquis> = FactRegistry::new(&[
    ProducerEntry::of::<Bindings>(),
    ProducerEntry::of::<UndefinedRefs>(),
    ProducerEntry::of::<ComponentUsages>(),
    ProducerEntry::of::<Reactivity>(),
]);

/// One drawn [`Croquis`] and the facts computed over it.
///
/// Holds a [`FactManager`] over [`CROQUIS_FACTS`], so every group is
/// produced at most once for the artifact however many consumers demand it.
/// Creating one allocates nothing; a group allocates only when a consumer's
/// demand first reaches it.
pub struct CroquisFacts<'c> {
    croquis: &'c Croquis,
    manager: FactManager<'static, Croquis>,
}

impl<'c> CroquisFacts<'c> {
    /// Facts over `croquis`, none computed yet.
    #[must_use]
    pub fn new(croquis: &'c Croquis) -> Self {
        Self {
            croquis,
            manager: FactManager::new(&CROQUIS_FACTS),
        }
    }

    /// The artifact the facts are computed over.
    #[must_use]
    pub fn croquis(&self) -> &'c Croquis {
        self.croquis
    }

    /// Compute `C`'s demand and return `C`'s view.
    ///
    /// # Panics
    ///
    /// When `C` demands a group [`CROQUIS_FACTS`] does not register — a
    /// static mistake in `C`'s const demand, never a data-dependent one.
    pub fn prepare<C: FactConsumer>(&mut self) -> FactView<'_> {
        match self.manager.prepare::<C>(self.croquis) {
            Ok(view) => view,
            Err(error) => panic!(
                "{} demands an unregistered Croquis fact group: {error:?}",
                C::NAME
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bindings, CROQUIS_FACTS, CroquisFacts, Demand, FactConsumer, FactGroup};
    use super::{ComponentUsages, FactError, Reactivity, UndefinedRefs};
    use crate::Croquis;
    use vize_davinci::fact::ids;

    struct ReadsBindings;
    impl FactConsumer for ReadsBindings {
        const NAME: &'static str = "test/reads-bindings";
        const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
    }

    #[test]
    fn the_registry_holds_the_allocated_ids() {
        assert_eq!(
            CROQUIS_FACTS.registered(),
            Demand::NONE
                .with(ids::BINDINGS)
                .with(ids::UNDEFINED_REFS)
                .with(ids::COMPONENT_USAGES)
                .with(ids::REACTIVITY)
        );
        assert_eq!(
            (
                Bindings::ID,
                UndefinedRefs::ID,
                ComponentUsages::ID,
                Reactivity::ID
            ),
            (
                ids::BINDINGS,
                ids::UNDEFINED_REFS,
                ids::COMPONENT_USAGES,
                ids::REACTIVITY
            )
        );
    }

    #[test]
    fn a_consumer_computes_only_its_demand() {
        let croquis = Croquis::new();
        let mut facts = CroquisFacts::new(&croquis);
        let view = facts.prepare::<ReadsBindings>();
        assert_eq!(view.get::<Bindings>().map(|table| table.len()), Ok(0));
        #[cfg(debug_assertions)]
        assert_eq!(
            view.get::<UndefinedRefs>().err(),
            Some(FactError::Undeclared {
                consumer: "test/reads-bindings",
                group: ids::UNDEFINED_REFS,
            })
        );
        facts.prepare::<ReadsBindings>();
        assert_eq!(facts.manager.computed(), Demand::NONE.with(Bindings::ID));
    }
}
