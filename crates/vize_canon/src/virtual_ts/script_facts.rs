//! Script bindings and undefined template names, read as fact groups.
//!
//! The checker still receives the drawn `Croquis`. These helpers are the
//! only way `virtual_ts` consults `Bindings` and `UndefinedRefs`, so the
//! struct fields stay producer storage.

use std::{cell::RefCell, rc::Rc};

use vize_croquis::facts::{
    Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup, UndefinedRefs,
};
use vize_croquis::{BindingType, Croquis, UndefinedRef};
use vize_l0::fact::FactTable;

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

struct CachedBindings {
    summary: *const Croquis,
    table: Rc<FactTable<Bindings>>,
}

thread_local! {
    /// The current synchronous projection's bindings. Batch generation is
    /// parallel across threads, while every individual projection is
    /// synchronous, so a lexical thread-local scope gives each SFC one table
    /// without locking or widening dozens of hot function signatures.
    static CACHED_BINDINGS: RefCell<Option<CachedBindings>> = const { RefCell::new(None) };
}

#[cfg(test)]
fn preparation_count(increment: bool) -> usize {
    thread_local! {
        static COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    COUNT.with(|count| {
        if increment {
            count.set(count.get() + 1);
        }
        count.get()
    })
}

/// Restores an outer projection's cache on normal return and unwinding.
struct BindingsScope(Option<CachedBindings>);

impl Drop for BindingsScope {
    fn drop(&mut self) {
        CACHED_BINDINGS.with_borrow_mut(|cache| {
            *cache = self.0.take();
        });
    }
}

fn prepare_bindings(summary: &Croquis) -> Option<Rc<FactTable<Bindings>>> {
    #[cfg(test)]
    preparation_count(true);
    vize_l0::micro_count!("canon.script_facts.prepare_bindings.calls", 1);
    let facts = CroquisFacts::new(summary);
    vize_l0::micro_span!(
        "canon.script_facts.prepare_bindings",
        facts
            .prepare_owned::<CanonBindings, Bindings>()
            .ok()
            .map(Rc::new)
    )
}

/// Run one virtual-TS projection with one bindings table for every consumer.
pub(super) fn with_cached_bindings<T>(summary: &Croquis, body: impl FnOnce() -> T) -> T {
    let current = CachedBindings {
        summary: std::ptr::from_ref(summary),
        table: prepare_bindings(summary).unwrap_or_else(|| Rc::new(FactTable::default())),
    };
    let previous = CACHED_BINDINGS.with_borrow_mut(|cache| cache.replace(current));
    let _scope = BindingsScope(previous);
    body()
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
    vize_l0::micro_count!("canon.script_facts.with_bindings.calls", 1);
    let cached = CACHED_BINDINGS.with_borrow(|cache| {
        cache
            .as_ref()
            .filter(|cached| std::ptr::eq(cached.summary, summary))
            .map(|cached| Rc::clone(&cached.table))
    });
    let table = cached.or_else(|| prepare_bindings(summary));
    body(ScriptBindings(table.as_deref()))
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

#[cfg(test)]
mod tests {
    use super::{binding_type, preparation_count, with_cached_bindings};
    use vize_croquis::{BindingType, Croquis};

    #[test]
    fn one_projection_prepares_bindings_once_and_nested_scopes_restore_the_owner() {
        let mut outer = Croquis::new();
        outer.bindings.add("outer", BindingType::SetupRef);
        let mut inner = Croquis::new();
        inner.bindings.add("inner", BindingType::SetupConst);
        let before = preparation_count(false);

        with_cached_bindings(&outer, || {
            for _ in 0..100 {
                assert_eq!(binding_type(&outer, "outer"), Some(BindingType::SetupRef));
            }
            with_cached_bindings(&inner, || {
                assert_eq!(binding_type(&inner, "inner"), Some(BindingType::SetupConst));
                assert_eq!(binding_type(&inner, "outer"), None);
            });
            assert_eq!(binding_type(&outer, "outer"), Some(BindingType::SetupRef));
        });

        assert_eq!(preparation_count(false) - before, 2);
    }
}
