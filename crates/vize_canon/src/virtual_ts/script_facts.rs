//! Script bindings and undefined template names, read as fact groups.
//!
//! The checker still receives the drawn `Croquis`. These helpers are the
//! only way `virtual_ts` consults `Bindings` and `UndefinedRefs`, so the
//! struct fields stay producer storage.

use vize_croquis::facts::{
    Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup, UndefinedRefs,
};
use vize_croquis::{BindingType, Croquis, UndefinedRef};
use vize_davinci::fact::FactTable;

struct CanonBindings;

impl FactConsumer for CanonBindings {
    const NAME: &'static str = "canon/virtual-ts-bindings";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

struct CanonUndefinedRefs;

impl FactConsumer for CanonUndefinedRefs {
    const NAME: &'static str = "canon/virtual-ts-undefined-refs";
    const DEMAND: Demand = Demand::NONE.with(UndefinedRefs::ID);
}

/// Borrowed `Bindings` table. Methods are inherent so callers do not import the trait.
///
/// `None` only if the fact manager refused the declared demand; every query
/// then answers as for a script without bindings.
#[derive(Clone, Copy)]
pub(super) struct ScriptBindings<'a>(Option<&'a FactTable<Bindings>>);

impl ScriptBindings<'_> {
    pub(super) fn is_script_setup(&self) -> bool {
        self.0.is_some_and(BindingsTable::is_script_setup)
    }

    pub(super) fn typed(&self) -> impl Iterator<Item = (&str, BindingType)> {
        self.0.into_iter().flat_map(BindingsTable::typed)
    }

    pub(super) fn spans(&self) -> impl Iterator<Item = (&str, (u32, u32))> {
        self.0.into_iter().flat_map(BindingsTable::spans)
    }

    pub(super) fn contains_binding(&self, name: &str) -> bool {
        self.0.is_some_and(|table| table.contains_binding(name))
    }

    pub(super) fn binding_type(&self, name: &str) -> Option<BindingType> {
        self.0.and_then(|table| table.binding_type(name))
    }

    pub(super) fn span(&self, name: &str) -> Option<(u32, u32)> {
        self.0.and_then(|table| table.span(name))
    }
}

pub(super) fn with_bindings<T>(summary: &Croquis, body: impl FnOnce(ScriptBindings<'_>) -> T) -> T {
    let mut facts = CroquisFacts::new(summary);
    let view = facts.prepare::<CanonBindings>();
    body(ScriptBindings(view.get::<Bindings>().ok()))
}

pub(super) fn binding_type(summary: &Croquis, name: &str) -> Option<BindingType> {
    with_bindings(summary, |bindings| bindings.binding_type(name))
}

pub(super) fn contains_binding(summary: &Croquis, name: &str) -> bool {
    with_bindings(summary, |bindings| bindings.contains_binding(name))
}

pub(super) fn is_script_setup(summary: &Croquis) -> bool {
    with_bindings(summary, |bindings| bindings.is_script_setup())
}

pub(super) fn has_typed_bindings(summary: &Croquis) -> bool {
    with_bindings(summary, |bindings| bindings.typed().next().is_some())
}

pub(super) fn binding_span(summary: &Croquis, name: &str) -> Option<(u32, u32)> {
    with_bindings(summary, |bindings| bindings.span(name))
}

/// Undefined template names in the drawer's walk order.
pub(super) fn undefined_refs(summary: &Croquis) -> Vec<UndefinedRef> {
    let mut facts = CroquisFacts::new(summary);
    let view = facts.prepare::<CanonUndefinedRefs>();
    view.get::<UndefinedRefs>()
        .map(|refs| {
            refs.iter()
                .map(|(_, reference)| reference.clone())
                .collect()
        })
        .unwrap_or_default()
}
