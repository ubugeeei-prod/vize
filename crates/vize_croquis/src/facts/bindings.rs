//! The `Bindings` fact group (P4-3a): script bindings, their definition
//! spans and props aliases, keyed by binding name.
//!
//! Folds three `Croquis` fields into one relation — `bindings` (name →
//! [`BindingType`] plus the script-setup flag), `binding_spans` (name →
//! definition span) and `bindings.props_aliases` (local → prop key) — so a
//! consumer that needs any of them demands one group. A name present in only
//! some of the fields keeps `None` in the others: the group is the exact
//! union, nothing is inferred.

use vize_carton::CompactString;
use vize_davinci::fact::{
    Demand, FactGroup, FactProducer, FactTable, FactTableBuilder, FactView, ids,
};
use vize_davinci::pass::AnalysisId;
use vize_relief::BindingType;

use crate::Croquis;

/// A `Bindings` key: the nullary script-setup marker, then one key per name.
///
/// The marker is a presence fact — the key exists exactly when the bindings
/// come from `<script setup>` — and sorts before every name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BindingKey {
    /// Present iff the bindings come from `<script setup>`.
    ScriptSetup,
    /// A binding name.
    Name(CompactString),
}

/// Everything the drawn artifact records about one binding name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindingFact {
    /// How the template and script see the binding (`Croquis.bindings`).
    pub kind: Option<BindingType>,
    /// Definition span in script offsets (`Croquis.binding_spans`).
    pub span: Option<(u32, u32)>,
    /// The prop key a destructured props local aliases (`props_aliases`).
    pub prop_key: Option<CompactString>,
}

/// The `Bindings` fact group.
pub struct Bindings;

impl FactGroup for Bindings {
    const ID: AnalysisId = ids::BINDINGS;
    const NAME: &'static str = "bindings";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = BindingKey;
    type Value = BindingFact;
}

impl FactProducer<Croquis> for Bindings {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let metadata = &croquis.bindings;
        let mut builder = FactTableBuilder::<Self>::with_capacity(
            metadata.bindings.len() + usize::from(metadata.is_script_setup),
        );
        if metadata.is_script_setup {
            builder.insert(BindingKey::ScriptSetup, BindingFact::default());
        }
        // Collect the union first: a builder keeps the last insert per key,
        // so each name must be inserted once with all three facets.
        let mut names: Vec<&CompactString> = metadata
            .bindings
            .keys()
            .chain(croquis.binding_spans.keys())
            .chain(metadata.props_aliases.keys())
            .collect();
        names.sort_unstable();
        names.dedup();
        for name in names {
            let fact = BindingFact {
                kind: metadata.bindings.get(name).copied(),
                span: croquis.binding_spans.get(name).copied(),
                prop_key: metadata.props_aliases.get(name).cloned(),
            };
            builder.insert(BindingKey::Name(name.clone()), fact);
        }
        builder.finish()
    }
}

/// Name-level reads over a `Bindings` table.
pub trait BindingsTable {
    /// Whether the bindings come from `<script setup>`.
    fn is_script_setup(&self) -> bool;
    /// The fact recorded for `name`, whatever facets it carries.
    fn fact(&self, name: &str) -> Option<&BindingFact>;
    /// The binding type of `name` (`Croquis.bindings.get`).
    fn binding_type(&self, name: &str) -> Option<BindingType>;
    /// Whether `name` is a binding (`Croquis.bindings.contains`).
    fn contains_binding(&self, name: &str) -> bool;
    /// The definition span of `name` (`Croquis.binding_spans.get`).
    fn span(&self, name: &str) -> Option<(u32, u32)>;
    /// Every typed binding, in ascending name order.
    fn typed(&self) -> impl Iterator<Item = (&str, BindingType)>;
    /// Every name with a definition span, in ascending name order.
    fn spans(&self) -> impl Iterator<Item = (&str, (u32, u32))>;
}

impl BindingsTable for FactTable<Bindings> {
    fn is_script_setup(&self) -> bool {
        self.contains_key(&BindingKey::ScriptSetup)
    }

    fn fact(&self, name: &str) -> Option<&BindingFact> {
        self.get(&BindingKey::Name(CompactString::new(name)))
    }

    fn binding_type(&self, name: &str) -> Option<BindingType> {
        self.fact(name).and_then(|fact| fact.kind)
    }

    fn contains_binding(&self, name: &str) -> bool {
        self.binding_type(name).is_some()
    }

    fn span(&self, name: &str) -> Option<(u32, u32)> {
        self.fact(name).and_then(|fact| fact.span)
    }

    fn typed(&self) -> impl Iterator<Item = (&str, BindingType)> {
        self.iter()
            .filter_map(|(key, fact)| match (key, fact.kind) {
                (BindingKey::Name(name), Some(kind)) => Some((name.as_str(), kind)),
                _ => None,
            })
    }

    fn spans(&self) -> impl Iterator<Item = (&str, (u32, u32))> {
        self.iter()
            .filter_map(|(key, fact)| match (key, fact.span) {
                (BindingKey::Name(name), Some(span)) => Some((name.as_str(), span)),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{BindingFact, BindingKey, Bindings, BindingsTable};
    use crate::Croquis;
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use vize_carton::CompactString;
    use vize_relief::BindingType;

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/bindings-reader";
        const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
    }

    #[test]
    fn the_group_is_the_exact_union_of_the_three_fields() {
        let mut croquis = Croquis::new();
        croquis.bindings.is_script_setup = true;
        croquis.bindings.add("count", BindingType::SetupRef);
        croquis.bindings.add("label", BindingType::PropsAliased);
        croquis
            .bindings
            .props_aliases
            .insert(CompactString::new("label"), CompactString::new("title"));
        croquis
            .binding_spans
            .insert(CompactString::new("count"), (6, 11));
        croquis
            .binding_spans
            .insert(CompactString::new("Only"), (40, 44));

        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<Bindings>().unwrap();
        let entries: Vec<(BindingKey, BindingFact)> = table
            .iter()
            .map(|(key, fact)| (key.clone(), fact.clone()))
            .collect();
        let name = |text: &str| BindingKey::Name(CompactString::new(text));
        assert_eq!(
            entries,
            vec![
                (BindingKey::ScriptSetup, BindingFact::default()),
                (
                    name("Only"),
                    BindingFact {
                        kind: None,
                        span: Some((40, 44)),
                        prop_key: None
                    }
                ),
                (
                    name("count"),
                    BindingFact {
                        kind: Some(BindingType::SetupRef),
                        span: Some((6, 11)),
                        prop_key: None
                    }
                ),
                (
                    name("label"),
                    BindingFact {
                        kind: Some(BindingType::PropsAliased),
                        span: None,
                        prop_key: Some(CompactString::new("title"))
                    }
                ),
            ]
        );
        assert!(table.is_script_setup());
        assert_eq!(
            (table.contains_binding("Only"), table.span("Only")),
            (false, Some((40, 44)))
        );
        let typed: Vec<(&str, BindingType)> = table.typed().collect();
        assert_eq!(
            typed,
            vec![
                ("count", BindingType::SetupRef),
                ("label", BindingType::PropsAliased)
            ]
        );
        let spans: Vec<(&str, (u32, u32))> = table.spans().collect();
        assert_eq!(spans, vec![("Only", (40, 44)), ("count", (6, 11))]);
    }
}
