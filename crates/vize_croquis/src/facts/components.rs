//! The `ComponentUsages` fact group (P4-3b): template component usages keyed
//! by the component's resolved identity.
//!
//! The key is the module the component was imported from plus the name that
//! module exports. `import { Foo as Bar } from './Foo.vue'` used as `<Bar />`
//! is `Foo` from `./Foo.vue`, not the local alias `Bar`. A component declared
//! in this file, or one that no import resolves, keeps `module: None` and the
//! name it is bound as — there is no alias to undo. This group is per file.
//! The cross-file render tree is a later group.

use vize_carton::CompactString;
use vize_davinci::fact::{Demand, FactGroup, FactProducer, FactTable, FactView, ids};
use vize_davinci::pass::AnalysisId;

use crate::Croquis;
use crate::croquis::ComponentRegistration;
use crate::naming::to_pascal_case;
use crate::scope::ScopeData;

/// A component as resolved in this file, independent of the local binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentIdentity {
    /// Module specifier the export was imported from.
    ///
    /// `None` when the component is declared in this file or no import resolves it.
    pub module: Option<CompactString>,
    /// Name the module exports, or the local declaration name when unresolved.
    pub export_name: CompactString,
}

/// One template use of a resolved component, in template order under its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentUsageSite {
    /// Start offset in the template.
    pub start: u32,
    /// End offset in the template.
    pub end: u32,
}

/// The `ComponentUsages` fact group.
pub struct ComponentUsages;

impl FactGroup for ComponentUsages {
    const ID: AnalysisId = ids::COMPONENT_USAGES;
    const NAME: &'static str = "component-usages";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ComponentIdentity;
    type Value = Vec<ComponentUsageSite>;
}

impl FactProducer<Croquis> for ComponentUsages {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let mut grouped: Vec<(ComponentIdentity, Vec<ComponentUsageSite>)> =
            Vec::with_capacity(croquis.component_usages.len());
        for usage in &croquis.component_usages {
            let identity = resolve(croquis, usage.name.as_str());
            let site = ComponentUsageSite {
                start: usage.start,
                end: usage.end,
            };
            if let Some((_, sites)) = grouped.iter_mut().find(|(key, _)| key == &identity) {
                sites.push(site);
            } else {
                grouped.push((identity, vec![site]));
            }
        }
        grouped.into_iter().collect()
    }
}

/// The identity a template tag renders: through an Options API registration,
/// then through a value import, preferring an exact local name over a
/// Pascal/kebab spelling of it.
fn resolve(croquis: &Croquis, tag: &str) -> ComponentIdentity {
    let bound = registration_local(croquis, tag).unwrap_or(tag);
    if let Some((module, export_name)) = imported_export(croquis, bound) {
        return ComponentIdentity {
            module: Some(CompactString::new(module)),
            export_name: CompactString::new(export_name),
        };
    }
    ComponentIdentity {
        module: None,
        export_name: CompactString::new(bound),
    }
}

fn registration_local<'a>(croquis: &'a Croquis, tag: &str) -> Option<&'a str> {
    let registrations = &croquis.component_registrations;
    exact_registration(registrations, tag)
        .or_else(|| {
            registrations
                .iter()
                .find(|registration| names_match(registration.name.as_str(), tag))
        })
        .map(|registration| registration.local_name.as_str())
}

fn exact_registration<'a>(
    registrations: &'a [ComponentRegistration],
    tag: &str,
) -> Option<&'a ComponentRegistration> {
    registrations
        .iter()
        .find(|registration| registration.name.as_str() == tag)
}

fn imported_export<'a>(croquis: &'a Croquis, local: &str) -> Option<(&'a str, &'a str)> {
    let mut fallback = None;
    for scope in croquis.scopes.iter() {
        let ScopeData::ExternalModule(data) = scope.data() else {
            continue;
        };
        if data.is_type_only {
            continue;
        }
        for export in &data.exports {
            if export.local_name.as_str() == local {
                return Some((data.source.as_str(), export.export_name.as_str()));
            }
            if fallback.is_none() && names_match(export.local_name.as_str(), local) {
                fallback = Some((data.source.as_str(), export.export_name.as_str()));
            }
        }
    }
    fallback
}

/// Vue matches a template tag to a binding in either spelling.
fn names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

#[cfg(test)]
mod tests {
    use super::ComponentUsages;
    use crate::drawer::{Drawer, DrawerOptions};
    use crate::facts::{CroquisFacts, Demand, FactConsumer, FactGroup};
    use vize_armature::parse;
    use vize_carton::Allocator;

    struct Reader;
    impl FactConsumer for Reader {
        const NAME: &'static str = "test/component-usages-reader";
        const DEMAND: Demand = Demand::NONE.with(ComponentUsages::ID);
    }

    #[test]
    fn aliased_import_records_the_exported_identity() {
        let allocator = Allocator::new();
        let (root, _errors) = parse(&allocator, "<Bar />");
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_script_setup("import { Foo as Bar } from './Foo.vue'\n");
        drawer.draw_template(&root);
        let croquis = drawer.finish();

        assert_eq!(croquis.component_usages.len(), 1);
        assert_eq!(croquis.component_usages[0].name.as_str(), "Bar");

        let mut facts = CroquisFacts::new(&croquis);
        let table = facts.prepare::<Reader>().get::<ComponentUsages>().unwrap();
        let recorded: Vec<_> = table
            .iter()
            .map(|(identity, sites)| {
                (
                    identity.module.as_deref(),
                    identity.export_name.as_str(),
                    sites.len(),
                )
            })
            .collect();
        assert_eq!(recorded, vec![(Some("./Foo.vue"), "Foo", 1)]);
        assert!(
            table
                .iter()
                .all(|(identity, _)| identity.export_name.as_str() != "Bar")
        );
    }
}
