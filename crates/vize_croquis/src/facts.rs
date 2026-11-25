//! Croquis products as fact groups (Davinci P4-3).
//!
//! Each tracker product a consumer reads becomes a fact group on the
//! [`vize_l0::analysis`] query surface: the tracker code stays the
//! population pass (semantic-engine.md #1), a registered producer turns its
//! output into a key-sorted [`AnalysisTable`], and a consumer reads the table
//! through a [`AnalysisView`] scoped to the demand it declared as const data
//! ([`AnalysisConsumer::DEMAND`]). The debug detector refuses every read outside
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
//! | [`UnusedBindings`] | binding name, with definition span | P4-3c |
//! | [`UndefinedRefs`] | walk-order ordinal | P4-3a |
//! | [`ComponentUsages`] | [`ComponentIdentity`] — module plus exported name | P4-3b |
//! | [`Reactivity`] | registration ordinal, then loss ordinal | P4-3d |
//! | [`ProvideInject`] | provide, inject, then composable ordinals | P4-3f |
//! | [`RaceConditions`] | risk ordinal | P4-3f |
//!
//! Every group carries a declarative specification and a naive evaluator in
//! [`spec`] (TS-34, the Polonius discipline).
//!
//! # Reading a group
//!
//! ```
//! use vize_croquis::Croquis;
//! use vize_croquis::facts::{
//!     BindingsTable, CroquisFacts, Demand, AnalysisConsumer, Analysis, bindings::Bindings,
//! };
//! use vize_relief::BindingType;
//!
//! struct CountRefs;
//! impl AnalysisConsumer for CountRefs {
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
pub mod provide_inject;
pub mod race;
pub mod reactivity;
pub mod spec;
pub mod undefined_refs;
pub mod unused_bindings;

pub use bindings::{BindingFact, BindingKey, Bindings, BindingsTable};
pub use components::{ComponentIdentity, ComponentUsages, GroupedComponentUse, component_identity};
pub use provide_inject::{
    ProvideInject, ProvideInjectFact, ProvideInjectKey, composable_calls, inject_entries,
    provide_entries,
};
pub use race::{RaceConditions, race_risks};
pub use reactivity::{
    Reactivity, ReactivityFact, ReactivityKey, SourceFact, reactivity_count, reactivity_has_losses,
    reactivity_is_reactive, reactivity_lookup, reactivity_losses, reactivity_sources,
};

mod access;
#[cfg(test)]
mod access_tests;
pub use access::{
    component_usage_list, component_usage_slice, used_component_contains, used_component_name_list,
    used_components_empty,
};
pub use undefined_refs::UndefinedRefs;
pub use unused_bindings::{UnusedBindingFact, UnusedBindings};
pub use vize_l0::analysis::{
    Analysis, AnalysisConsumer, AnalysisError, AnalysisManager, AnalysisRegistry, AnalysisTable,
    AnalysisView, Demand, ProducerEntry,
};

/// Compatibility name retained for one minor release; use [`Analysis`].
#[deprecated(note = "use Analysis; retained for one minor release (#13707)")]
pub use vize_l0::analysis::Analysis as FactGroup;
/// Compatibility name retained for one minor release; use [`AnalysisConsumer`].
#[deprecated(note = "use AnalysisConsumer; retained for one minor release (#13707)")]
pub use vize_l0::analysis::AnalysisConsumer as FactConsumer;
/// Compatibility name retained for one minor release; use [`AnalysisError`].
#[deprecated(note = "use AnalysisError; retained for one minor release (#13707)")]
pub use vize_l0::analysis::AnalysisError as FactError;
/// Compatibility name retained for one minor release; use [`AnalysisManager`].
#[deprecated(note = "use AnalysisManager; retained for one minor release (#13707)")]
pub type FactManager<'r, A> = AnalysisManager<'r, A>;
/// Compatibility name retained for one minor release; use [`AnalysisRegistry`].
#[deprecated(note = "use AnalysisRegistry; retained for one minor release (#13707)")]
pub type FactRegistry<A> = AnalysisRegistry<A>;
/// Compatibility name retained for one minor release; use [`AnalysisTable`].
#[deprecated(note = "use AnalysisTable; retained for one minor release (#13707)")]
pub type FactTable<G> = AnalysisTable<G>;
/// Compatibility name retained for one minor release; use [`AnalysisView`].
#[deprecated(note = "use AnalysisView; retained for one minor release (#13707)")]
pub type FactView<'a> = AnalysisView<'a>;

use crate::Croquis;

/// Every Croquis fact group, stratified at compile time.
pub const CROQUIS_FACTS: AnalysisRegistry<Croquis> = AnalysisRegistry::new(&[
    ProducerEntry::of::<Bindings>(),
    ProducerEntry::of::<UndefinedRefs>(),
    ProducerEntry::of::<UnusedBindings>(),
    ProducerEntry::of::<ComponentUsages>(),
    ProducerEntry::of::<Reactivity>(),
    ProducerEntry::of::<ProvideInject>(),
    ProducerEntry::of::<RaceConditions>(),
]);

/// One drawn [`Croquis`] and the facts computed over it.
///
/// Holds a [`AnalysisManager`] over [`CROQUIS_FACTS`], so every group is
/// produced at most once for the artifact however many consumers demand it.
/// Creating one allocates nothing; a group allocates only when a consumer's
/// demand first reaches it.
pub struct CroquisFacts<'c> {
    croquis: &'c Croquis,
    manager: AnalysisManager<'static, Croquis>,
}

impl<'c> CroquisFacts<'c> {
    /// Facts over `croquis`, none computed yet.
    #[must_use]
    pub fn new(croquis: &'c Croquis) -> Self {
        Self {
            croquis,
            manager: AnalysisManager::new(&CROQUIS_FACTS),
        }
    }

    /// The artifact the facts are computed over.
    #[must_use]
    pub fn croquis(&self) -> &'c Croquis {
        self.croquis
    }

    /// Compute `C`'s demand and return `C`'s view.
    ///
    /// When `C` demands a group [`CROQUIS_FACTS`] does not register (a static
    /// mistake in `C`'s const demand, never a data-dependent one), nothing is
    /// computed for it and [`AnalysisView::get`] reports the group as not computed.
    pub fn prepare<C: AnalysisConsumer>(&mut self) -> AnalysisView<'_> {
        // `prepare` returns exactly `view::<C>()` on success; on failure the
        // view still answers, with an error for every group it lacks.
        let _ = self.manager.prepare::<C>(self.croquis);
        self.manager.view::<C>()
    }

    /// Compute `C`'s demand and transfer one declared table to the caller.
    ///
    /// Consuming the manager makes the table suitable for a longer-lived,
    /// read-only query scope without cloning its keys and values.
    pub fn prepare_owned<C: AnalysisConsumer, G: Analysis>(
        self,
    ) -> Result<AnalysisTable<G>, AnalysisError> {
        self.manager.prepare_owned::<C, G>(self.croquis)
    }
}

#[cfg(test)]
mod tests {
    use super::{Analysis, AnalysisConsumer, Bindings, CROQUIS_FACTS, CroquisFacts, Demand};
    use super::{
        AnalysisError, ComponentUsages, ProvideInject, RaceConditions, Reactivity, UndefinedRefs,
        UnusedBindings,
    };
    use crate::Croquis;
    use vize_l0::analysis::ids;

    struct ReadsBindings;
    impl AnalysisConsumer for ReadsBindings {
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
                .with(ids::UNUSED_BINDINGS)
                .with(ids::COMPONENT_USAGES)
                .with(ids::REACTIVITY)
                .with(ids::PROVIDE_INJECT)
                .with(ids::RACE_CONDITIONS)
        );
        assert_eq!(
            (
                Bindings::ID,
                UndefinedRefs::ID,
                UnusedBindings::ID,
                ComponentUsages::ID,
                Reactivity::ID,
                ProvideInject::ID,
                RaceConditions::ID,
            ),
            (
                ids::BINDINGS,
                ids::UNDEFINED_REFS,
                ids::UNUSED_BINDINGS,
                ids::COMPONENT_USAGES,
                ids::REACTIVITY,
                ids::PROVIDE_INJECT,
                ids::RACE_CONDITIONS,
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
            Some(AnalysisError::Undeclared {
                consumer: "test/reads-bindings",
                group: ids::UNDEFINED_REFS,
            })
        );
        facts.prepare::<ReadsBindings>();
        assert_eq!(facts.manager.computed(), Demand::NONE.with(Bindings::ID));
    }
}
