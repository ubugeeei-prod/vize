//! The `Reactivity` fact group (P4-3d).
//!
//! Each `ReactiveKind` source becomes one row of the P3-2 lattice in
//! `vize_impeto`: this module maps the kind onto a [`SourceKind`], and
//! [`evaluate_binding`](vize_impeto::lattice::evaluate_binding) is the only
//! classifier. Losses are copied in tracker order so a reader can leave
//! `Croquis.reactivity` without moving a diagnostic.

use crate::Croquis;
use crate::reactivity::{ReactiveKind, ReactivityLoss};
use vize_carton::{CompactString, Span};
use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactProducer, FactTable, FactTableBuilder, FactView, ids,
};
use vize_davinci::pass::AnalysisId;
use vize_impeto::lattice::{
    BindingId, EffectSet, ReactivityClass, SourceKind, Verdict, evaluate_binding,
};

/// A source row, or one loss in tracker order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReactivityKey {
    /// Registration ordinal. Duplicate names stay; lookup keeps the last.
    Source(u32),
    /// Loss ordinal.
    Loss(u32),
}

/// One lattice row, or one unchanged loss.
#[derive(Debug, Clone, PartialEq)]
pub enum ReactivityFact {
    /// A reactive source classified by the S3 lattice.
    Source(SourceFact),
    /// A reactivity loss, in the tracker's order.
    Loss(ReactivityLoss),
}

/// A source mapped onto the lattice's value axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFact {
    /// Binding name. The last registration of a name wins `lookup`.
    pub name: CompactString,
    /// The Vue kind the tracker recorded.
    pub kind: ReactiveKind,
    /// Declaration offset in script coordinates.
    pub declaration_offset: u32,
    /// Lattice value. Proven only when [`Self::verdict`] is proven.
    pub class: ReactivityClass,
    /// Epistemic axis. `readonly` stays unknown: the wrapped value is unseen.
    pub verdict: Verdict,
    /// Effects the kind constructor fed the lattice.
    pub effects: EffectSet,
}

/// The `Reactivity` fact group.
pub struct Reactivity;

impl FactGroup for Reactivity {
    const ID: AnalysisId = ids::REACTIVITY;
    const NAME: &'static str = "reactivity";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ReactivityKey;
    type Value = ReactivityFact;
}

impl FactProducer<Croquis> for Reactivity {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let sources = croquis.reactivity.sources();
        let losses = croquis.reactivity.losses();
        let mut builder = FactTableBuilder::<Self>::with_capacity(sources.len() + losses.len());
        for (ordinal, source) in sources.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            let input = source_kind(source.kind).binding_input(
                BindingId::new(ordinal),
                Span::new(source.declaration_offset, source.declaration_offset),
            );
            let fact = evaluate_binding(input);
            builder.insert(
                ReactivityKey::Source(ordinal),
                ReactivityFact::Source(SourceFact {
                    name: source.name.clone(),
                    kind: source.kind,
                    declaration_offset: source.declaration_offset,
                    class: fact.class,
                    verdict: fact.verdict,
                    effects: fact.effects,
                }),
            );
        }
        for (ordinal, loss) in losses.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            builder.insert(
                ReactivityKey::Loss(ordinal),
                ReactivityFact::Loss(loss.clone()),
            );
        }
        builder.finish()
    }
}

/// `ReactiveKind` onto the lattice constructor. Discriminants match
/// [`SourceKind`] so a reordered kind fails the spec's tag check.
fn source_kind(kind: ReactiveKind) -> SourceKind {
    match kind {
        ReactiveKind::Ref => SourceKind::Ref,
        ReactiveKind::ShallowRef => SourceKind::ShallowRef,
        ReactiveKind::Reactive => SourceKind::Reactive,
        ReactiveKind::ShallowReactive => SourceKind::ShallowReactive,
        ReactiveKind::Computed => SourceKind::Computed,
        ReactiveKind::Readonly => SourceKind::Readonly,
        ReactiveKind::ShallowReadonly => SourceKind::ShallowReadonly,
        ReactiveKind::ToRef => SourceKind::ToRef,
        ReactiveKind::ToRefs => SourceKind::ToRefs,
    }
}

struct ReactivityReader;

impl FactConsumer for ReactivityReader {
    const NAME: &'static str = "croquis/reactivity";
    const DEMAND: Demand = Demand::NONE.with(Reactivity::ID);
}

fn with_table<T>(croquis: &Croquis, read: impl FnOnce(&FactTable<Reactivity>) -> T) -> T {
    let mut facts = super::CroquisFacts::new(croquis);
    let view = facts.prepare::<ReactivityReader>();
    match view.get::<Reactivity>() {
        Ok(table) => read(table),
        Err(_) => read(&FactTable::default()),
    }
}

/// Sources in registration order, including a name registered twice.
#[must_use]
pub fn reactivity_sources(croquis: &Croquis) -> Vec<SourceFact> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ReactivityFact::Source(source) => Some(source.clone()),
                ReactivityFact::Loss(_) => None,
            })
            .collect()
    })
}

/// The last registration of `name`.
#[must_use]
pub fn reactivity_lookup(croquis: &Croquis, name: &str) -> Option<SourceFact> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ReactivityFact::Source(source) if source.name.as_str() == name => {
                    Some(source.clone())
                }
                _ => None,
            })
            .last()
    })
}

/// Losses in tracker order.
#[must_use]
pub fn reactivity_losses(croquis: &Croquis) -> Vec<ReactivityLoss> {
    with_table(croquis, |table| {
        table
            .iter()
            .filter_map(|(_, fact)| match fact {
                ReactivityFact::Loss(loss) => Some(loss.clone()),
                ReactivityFact::Source(_) => None,
            })
            .collect()
    })
}

/// Registration count, not the count of distinct names.
#[must_use]
pub fn reactivity_count(croquis: &Croquis) -> usize {
    reactivity_sources(croquis).len()
}

/// Whether `name` was registered.
#[must_use]
pub fn reactivity_is_reactive(croquis: &Croquis, name: &str) -> bool {
    reactivity_lookup(croquis, name).is_some()
}

/// Whether the tracker recorded a loss.
#[must_use]
pub fn reactivity_has_losses(croquis: &Croquis) -> bool {
    with_table(croquis, |table| {
        table
            .iter()
            .any(|(_, fact)| matches!(fact, ReactivityFact::Loss(_)))
    })
}

#[cfg(test)]
mod tests {
    use super::{Reactivity, reactivity_lookup, reactivity_losses, reactivity_sources};
    use crate::drawer::{Drawer, DrawerOptions};
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use crate::reactivity::ReactiveKind;
    use vize_impeto::lattice::{ReactivityClass, SourceKind, Verdict};

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/reactivity-reader";
        const DEMAND: Demand = Demand::NONE.with(Reactivity::ID);
    }

    #[test]
    fn kind_tags_match_the_lattice_constructor() {
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
        for kind in kinds {
            assert_eq!(super::source_kind(kind) as u8, kind as u8);
        }
        assert_eq!(SourceKind::ToRefs as u8, 8);
    }

    #[test]
    fn a_ref_is_a_proven_reactive_lattice_row_and_losses_round_trip() {
        let mut drawer = Drawer::with_options(DrawerOptions::for_lint());
        drawer.draw_script(
            r#"
            const state = reactive({ count: 0, name: 'test' })
            const { count, name } = state
            const countRef = ref(0)
            const value = countRef.value
            "#,
        );
        let croquis = drawer.finish();
        assert!(
            croquis.reactivity.losses().len() >= 2,
            "the fixture must record losses"
        );

        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<Reactivity>().unwrap();
        assert!(table.len() >= 4);

        let sources = reactivity_sources(&croquis);
        let count_ref = sources
            .iter()
            .find(|source| source.name.as_str() == "countRef")
            .expect("countRef");
        assert_eq!(count_ref.kind, ReactiveKind::Ref);
        assert_eq!(count_ref.class, ReactivityClass::Reactive);
        assert_eq!(count_ref.verdict, Verdict::Proven);
        assert_eq!(
            reactivity_lookup(&croquis, "countRef").map(|source| source.kind),
            Some(ReactiveKind::Ref)
        );

        let state = reactivity_lookup(&croquis, "state").expect("state");
        assert_eq!(state.kind, ReactiveKind::Reactive);
        assert_eq!(state.class, ReactivityClass::Reactive);
        assert_eq!(state.verdict, Verdict::Proven);

        assert_eq!(reactivity_losses(&croquis), croquis.reactivity.losses());
    }
}
