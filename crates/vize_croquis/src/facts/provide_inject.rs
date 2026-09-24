//! The `ProvideInject` fact group (P4-3f).
//!
//! Provides, injects, and composable calls in tracker order. Cross-file
//! pairing reads this table; it does not walk `Croquis.provide_inject`.

use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactTable, FactTableBuilder, FactView, ids,
};
use vize_davinci::pass::AnalysisId;

use crate::Croquis;
use crate::provide::{ComposableCall, InjectEntry, ProvideEntry};

/// One tracker row.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProvideInjectKey {
    /// `provides` ordinal.
    Provide(u32),
    /// `injects` ordinal.
    Inject(u32),
    /// `composables` ordinal.
    Composable(u32),
}

/// The row stored at [`ProvideInjectKey`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvideInjectFact {
    /// A `provide()` call.
    Provide(ProvideEntry),
    /// An `inject()` call.
    Inject(InjectEntry),
    /// A top-level composable call.
    Composable(ComposableCall),
}

/// The `ProvideInject` fact group.
pub struct ProvideInject;

impl FactGroup for ProvideInject {
    const ID: AnalysisId = ids::PROVIDE_INJECT;
    const NAME: &'static str = "provide-inject";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ProvideInjectKey;
    type Value = ProvideInjectFact;
}

impl FactProducer<Croquis> for ProvideInject {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let tracker = &croquis.provide_inject;
        let mut builder = FactTableBuilder::<Self>::with_capacity(
            tracker.provides().len() + tracker.injects().len() + tracker.composables().len(),
        );
        for (ordinal, provide) in tracker.provides().iter().enumerate() {
            builder.insert(
                ProvideInjectKey::Provide(ordinal_of(ordinal)),
                ProvideInjectFact::Provide(provide.clone()),
            );
        }
        for (ordinal, inject) in tracker.injects().iter().enumerate() {
            builder.insert(
                ProvideInjectKey::Inject(ordinal_of(ordinal)),
                ProvideInjectFact::Inject(inject.clone()),
            );
        }
        for (ordinal, call) in tracker.composables().iter().enumerate() {
            builder.insert(
                ProvideInjectKey::Composable(ordinal_of(ordinal)),
                ProvideInjectFact::Composable(call.clone()),
            );
        }
        builder.finish()
    }
}

fn ordinal_of(ordinal: usize) -> u32 {
    u32::try_from(ordinal).unwrap_or(u32::MAX)
}

struct ProvideInjectReader;

impl FactConsumer for ProvideInjectReader {
    const NAME: &'static str = "croquis/provide-inject";
    const DEMAND: Demand = Demand::NONE.with(ProvideInject::ID);
}

fn with_table<T>(croquis: &Croquis, read: impl FnOnce(&FactTable<ProvideInject>) -> T) -> T {
    let mut facts = super::CroquisFacts::new(croquis);
    match facts
        .prepare::<ProvideInjectReader>()
        .get::<ProvideInject>()
    {
        Ok(table) => read(table),
        Err(_) => read(&FactTable::default()),
    }
}

/// `provide()` calls in tracker order.
#[must_use]
pub fn provide_entries(croquis: &Croquis) -> Vec<ProvideEntry> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ProvideInjectFact::Provide(provide) => Some(provide.clone()),
                _ => None,
            })
            .collect()
    })
}

/// `inject()` calls in tracker order.
#[must_use]
pub fn inject_entries(croquis: &Croquis) -> Vec<InjectEntry> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ProvideInjectFact::Inject(inject) => Some(inject.clone()),
                _ => None,
            })
            .collect()
    })
}

/// Top-level composable calls in tracker order.
#[must_use]
pub fn composable_calls(croquis: &Croquis) -> Vec<ComposableCall> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ProvideInjectFact::Composable(call) => Some(call.clone()),
                _ => None,
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::{ProvideInject, inject_entries, provide_entries};
    use crate::drawer::{Drawer, DrawerOptions};
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use crate::provide::ProvideKey;
    use vize_carton::CompactString;

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/provide-inject-reader";
        const DEMAND: Demand = Demand::NONE.with(ProvideInject::ID);
    }

    #[test]
    fn provide_and_inject_rows_follow_the_tracker() {
        let mut drawer = Drawer::with_options(DrawerOptions::for_lint());
        drawer.draw_script(
            r#"
            const theme = ref('dark')
            provide('theme', theme)
            const color = inject('theme')
            "#,
        );
        let croquis = drawer.finish();
        assert!(!croquis.provide_inject.provides().is_empty());
        assert!(!croquis.provide_inject.injects().is_empty());

        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<ProvideInject>().unwrap();
        assert!(table.len() >= 2);
        assert_eq!(provide_entries(&croquis), croquis.provide_inject.provides());
        assert_eq!(inject_entries(&croquis), croquis.provide_inject.injects());
        assert_eq!(
            inject_entries(&croquis)[0].key,
            ProvideKey::String(CompactString::new("theme"))
        );
    }
}
