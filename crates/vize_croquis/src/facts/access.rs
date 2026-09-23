//! Readers and test writers for component usages.
//!
//! Field storage stays inside this crate. Everyone else asks the
//! `ComponentUsages` group, or records a usage through these methods.

use vize_carton::CompactString;

use crate::Croquis;
use crate::croquis::ComponentUsage;
use crate::facts::{
    ComponentUsages, CroquisFacts, Demand, FactConsumer, FactGroup, GroupedComponentUse,
};

struct ComponentUsageReader;

impl FactConsumer for ComponentUsageReader {
    const NAME: &'static str = "croquis/component-usages";
    const DEMAND: Demand = Demand::NONE.with(ComponentUsages::ID);
}

fn with_uses<T>(croquis: &Croquis, read: impl FnOnce(&[GroupedComponentUse]) -> T) -> T {
    let mut facts = CroquisFacts::new(croquis);
    let view = facts.prepare::<ComponentUsageReader>();
    let table = view
        .get::<ComponentUsages>()
        .expect("component usage demand");
    let uses: Vec<GroupedComponentUse> = table
        .iter()
        .flat_map(|(_, uses)| uses.iter().cloned())
        .collect();
    read(&uses)
}

/// Template component usages, in source order.
pub fn component_usage_list(croquis: &Croquis) -> Vec<ComponentUsage> {
    with_uses(croquis, |uses| {
        let mut records: Vec<(u32, ComponentUsage)> = uses
            .iter()
            .filter_map(|use_| {
                use_.usage
                    .clone()
                    .map(|usage| (use_.usage_index.unwrap_or(u32::MAX), usage))
            })
            .collect();
        records.sort_by_key(|(index, _)| *index);
        records.into_iter().map(|(_, usage)| usage).collect()
    })
}

/// Names from the used-component set, in that set's iteration order,
/// then usage names the set does not contain.
pub fn used_component_name_list(croquis: &Croquis) -> Vec<CompactString> {
    with_uses(croquis, |uses| {
        let mut named: Vec<(u32, CompactString)> = Vec::new();
        let mut extra: Vec<(u32, CompactString)> = Vec::new();
        for use_ in uses {
            if let Some(ordinal) = use_.name_ordinal {
                if named.iter().any(|(_, tag)| tag == use_.tag) {
                    continue;
                }
                named.push((ordinal, use_.tag.clone()));
            } else if !extra.iter().any(|(_, tag)| tag == use_.tag)
                && !named.iter().any(|(_, tag)| tag == use_.tag)
            {
                extra.push((use_.usage_index.unwrap_or(u32::MAX), use_.tag.clone()));
            }
        }
        named.sort_by_key(|(ordinal, _)| *ordinal);
        extra.sort_by_key(|(ordinal, _)| *ordinal);
        named.append(&mut extra);
        named.into_iter().map(|(_, tag)| tag).collect()
    })
}

pub fn used_components_empty(croquis: &Croquis) -> bool {
    used_component_name_list(croquis).is_empty()
}

pub fn used_component_contains(croquis: &Croquis, name: &str) -> bool {
    used_component_name_list(croquis)
        .iter()
        .any(|tag| tag.as_str() == name)
}

impl Croquis {
    pub fn note_used_component(&mut self, name: impl AsRef<str>) {
        self.used_components
            .insert(CompactString::new(name.as_ref()));
    }

    pub fn note_component_usage(&mut self, usage: ComponentUsage) {
        self.component_usages.push(usage);
    }
}
