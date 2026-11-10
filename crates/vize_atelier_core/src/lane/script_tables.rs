//! The `<script setup>` facts the template transform reads beyond the
//! binding metadata: the reactivity API behind a binding, and the names that
//! register a component.

use crate::options::BindingMetadata;
use vize_croquis::reactivity::ReactiveKind;
use vize_croquis::{BindingType, Croquis};
use vize_l0::CompactString;
pub use vize_l2::framework::vue::ReactiveSource;

/// Reactive sources and component registrations of a `<script setup>`, by
/// name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptTables {
    /// Sorted by name, one entry per name.
    reactive: std::vec::Vec<(CompactString, ReactiveSource)>,
    /// Sorted, without duplicates.
    components: std::vec::Vec<CompactString>,
}

impl ScriptTables {
    /// Tables over `reactive` sources and `components` registrations. A
    /// later source for a name replaces an earlier one.
    #[must_use]
    pub fn new<'n>(
        reactive: impl IntoIterator<Item = (&'n str, ReactiveSource)>,
        components: impl IntoIterator<Item = &'n str>,
    ) -> Self {
        let mut reactive: std::vec::Vec<_> = reactive
            .into_iter()
            .map(|(name, source)| (CompactString::new(name), source))
            .collect();
        reactive.reverse();
        reactive.sort_by(|(left, _), (right, _)| left.cmp(right));
        reactive.dedup_by(|(later, _), (earlier, _)| later == earlier);
        let mut components: std::vec::Vec<_> =
            components.into_iter().map(CompactString::new).collect();
        components.sort_unstable();
        components.dedup();
        Self {
            reactive,
            components,
        }
    }

    /// The reactivity API whose result initializes `name`, if any.
    #[must_use]
    pub fn reactive_source(&self, name: &str) -> Option<ReactiveSource> {
        self.reactive
            .binary_search_by(|(known, _)| known.as_str().cmp(name))
            .ok()
            .and_then(|index| self.reactive.get(index))
            .map(|(_, source)| *source)
    }

    /// Each reactive binding's source, sorted by name.
    pub fn reactive(&self) -> impl Iterator<Item = (&str, ReactiveSource)> {
        self.reactive
            .iter()
            .map(|(name, source)| (name.as_str(), *source))
    }

    /// Whether `name` registers a component.
    #[must_use]
    pub fn registers_component(&self, name: &str) -> bool {
        self.components
            .binary_search_by(|known| known.as_str().cmp(name))
            .is_ok()
    }

    /// The names that register a component, sorted.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.components.iter().map(CompactString::as_str)
    }

    /// Whether `metadata` binds every name that registers a component, so a
    /// lane that resolves components from the metadata alone agrees.
    #[must_use]
    pub fn components_bound_in(&self, metadata: &BindingMetadata) -> bool {
        self.components
            .iter()
            .all(|name| metadata.bindings.contains_key(name.as_str()))
    }

    /// The tables a Croquis summary answers: its reactivity tracker's
    /// registrations, and its used components plus `SetupConst` bindings.
    #[must_use]
    pub fn from_croquis(croquis: &Croquis) -> Self {
        let reactive = croquis
            .reactivity
            .sources()
            .iter()
            .map(|source| (source.name.as_str(), reactive_source(source.kind)));
        let consts = croquis
            .bindings
            .iter()
            .filter(|(_, kind)| *kind == BindingType::SetupConst)
            .map(|(name, _)| name);
        let drawn = vize_croquis::facts::used_component_name_list(croquis);
        let components = drawn.iter().map(CompactString::as_str).chain(consts);
        Self::new(reactive, components)
    }
}

fn reactive_source(kind: ReactiveKind) -> ReactiveSource {
    match kind {
        ReactiveKind::Ref => ReactiveSource::Ref,
        ReactiveKind::ShallowRef => ReactiveSource::ShallowRef,
        ReactiveKind::Reactive => ReactiveSource::Reactive,
        ReactiveKind::ShallowReactive => ReactiveSource::ShallowReactive,
        ReactiveKind::Computed => ReactiveSource::Computed,
        ReactiveKind::Readonly => ReactiveSource::Readonly,
        ReactiveKind::ShallowReadonly => ReactiveSource::ShallowReadonly,
        ReactiveKind::ToRef => ReactiveSource::ToRef,
        ReactiveKind::ToRefs => ReactiveSource::ToRefs,
    }
}

#[cfg(test)]
mod tests {
    use super::{ReactiveSource, ScriptTables};
    use vize_croquis::reactivity::ReactiveKind;
    use vize_croquis::{BindingType, Croquis};

    #[test]
    fn a_later_source_replaces_an_earlier_one() {
        let tables = ScriptTables::new(
            [
                ("count", ReactiveSource::Ref),
                ("state", ReactiveSource::Reactive),
                ("count", ReactiveSource::Readonly),
            ],
            ["Child", "Child"],
        );
        assert_eq!(
            tables.reactive_source("count"),
            Some(ReactiveSource::Readonly)
        );
        assert_eq!(tables.reactive_source("other"), None);
        assert_eq!(
            tables.reactive().collect::<Vec<_>>(),
            [
                ("count", ReactiveSource::Readonly),
                ("state", ReactiveSource::Reactive)
            ]
        );
        assert_eq!(tables.components().collect::<Vec<_>>(), ["Child"]);
        assert!(tables.registers_component("Child") && !tables.registers_component("child"));
    }

    /// Every lookup equals the Croquis read it replaces.
    #[test]
    fn croquis_tables_answer_like_the_summary() {
        let kinds = [
            ReactiveKind::Ref,
            ReactiveKind::ShallowRef,
            ReactiveKind::Reactive,
            ReactiveKind::ShallowReactive,
            ReactiveKind::Computed,
            ReactiveKind::Readonly,
            ReactiveKind::ShallowReadonly,
            ReactiveKind::ToRef,
            ReactiveKind::ToRefs,
        ];
        let mut croquis = Croquis::default();
        for (offset, kind) in (0..).zip(kinds) {
            croquis
                .reactivity
                .register(vize_l0::cstr!("k{offset}"), kind, offset);
        }
        croquis
            .reactivity
            .register("k0".into(), ReactiveKind::Computed, 9);
        croquis.bindings.add("Const", BindingType::SetupConst);
        croquis.bindings.add("Let", BindingType::SetupLet);
        croquis.note_used_component("drawn");
        let tables = ScriptTables::from_croquis(&croquis);
        for name in ["k0", "k1", "k2", "k3", "k4", "k5", "k6", "k7", "k8", "k9"] {
            let expected = croquis.reactivity.lookup(name).map(|source| source.kind);
            let actual = tables.reactive_source(name);
            assert_eq!(
                actual.map(ReactiveSource::reads_value),
                expected.map(ReactiveKind::needs_value_access),
                "{name}"
            );
        }
        for name in ["Const", "Let", "drawn", "other"] {
            assert_eq!(
                tables.registers_component(name),
                croquis.is_component_registered(name),
                "{name}"
            );
        }
    }
}
