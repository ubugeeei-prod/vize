//! The storage readers must answer exactly as the `ComponentUsages` sites.

use vize_carton::{Allocator, CompactString};

use super::{
    component_usage_list, component_usage_slice, used_component_contains, used_component_name_list,
    used_components_empty,
};
use crate::facts::{
    ComponentUsages, CroquisFacts, Demand, FactConsumer, FactGroup, GroupedComponentUse,
};
use crate::{Croquis, Drawer, DrawerOptions};

struct Reader;

impl FactConsumer for Reader {
    const NAME: &'static str = "croquis/component-usages-test";
    const DEMAND: Demand = Demand::NONE.with(ComponentUsages::ID);
}

fn sites(croquis: &Croquis) -> Vec<GroupedComponentUse> {
    let mut facts = CroquisFacts::new(croquis);
    let view = facts.prepare::<Reader>();
    view.get::<ComponentUsages>()
        .map(|table| {
            table
                .iter()
                .flat_map(|(_, uses)| uses.iter().cloned())
                .collect()
        })
        .unwrap_or_default()
}

fn grouped_usages(croquis: &Croquis) -> Vec<crate::croquis::ComponentUsage> {
    let mut records: Vec<_> = sites(croquis)
        .into_iter()
        .filter_map(|site| {
            site.usage
                .map(|usage| (site.usage_index.unwrap_or(u32::MAX), usage))
        })
        .collect();
    records.sort_by_key(|(index, _)| *index);
    records.into_iter().map(|(_, usage)| usage).collect()
}

fn grouped_names(croquis: &Croquis) -> Vec<CompactString> {
    let mut named: Vec<(u32, CompactString)> = Vec::new();
    let mut extra: Vec<(u32, CompactString)> = Vec::new();
    for site in sites(croquis) {
        if let Some(ordinal) = site.name_ordinal {
            if !named.iter().any(|(_, tag)| *tag == site.tag) {
                named.push((ordinal, site.tag));
            }
        } else if !extra.iter().any(|(_, tag)| *tag == site.tag)
            && !named.iter().any(|(_, tag)| *tag == site.tag)
        {
            extra.push((site.usage_index.unwrap_or(u32::MAX), site.tag));
        }
    }
    named.sort_by_key(|(ordinal, _)| *ordinal);
    extra.sort_by_key(|(index, _)| *index);
    named.append(&mut extra);
    named.into_iter().map(|(_, tag)| tag).collect()
}

fn draw(script: &str, template: &str) -> Croquis {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);
    drawer.finish()
}

fn assert_matches_group(croquis: &Croquis) {
    assert_eq!(component_usage_slice(croquis), grouped_usages(croquis));
    assert_eq!(component_usage_list(croquis), grouped_usages(croquis));
    let names = grouped_names(croquis);
    assert_eq!(used_component_name_list(croquis), names);
    assert_eq!(used_components_empty(croquis), names.is_empty());
    for probe in ["Card", "my-card", "MyCard", "Bare", "Missing", "Extra"] {
        assert_eq!(
            used_component_contains(croquis, probe),
            names.iter().any(|name| name == probe),
            "{probe}"
        );
    }
}

#[test]
fn storage_readers_match_component_usage_sites() {
    let script = "import Card from './Card.vue'\nimport { Panel as MyCard } from './ui'\nimport Alias from './Card.vue'\n";
    let template = "<div><Card :a=\"1\" /><my-card /><MyCard @x=\"() => 1\"><template #default>a</template></MyCard><Card /><Alias /><Bare /></div>";
    let mut croquis = draw(script, template);
    assert!(croquis.component_usages.len() >= 5);
    assert_matches_group(&croquis);

    croquis.note_used_component("Missing");
    let mut extra = croquis.component_usages[0].clone();
    extra.name = CompactString::new("Extra");
    croquis.note_component_usage(extra.clone());
    croquis.note_component_usage(extra);
    assert_matches_group(&croquis);

    assert_matches_group(&Croquis::default());
    let mut names_only = Croquis::default();
    names_only.note_used_component("Card");
    assert_matches_group(&names_only);
}
