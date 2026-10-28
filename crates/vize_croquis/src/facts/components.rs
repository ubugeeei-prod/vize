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
use vize_l0::fact::{Demand, FactGroup, FactProducer, FactTable, FactView, ids};
use vize_l0::pass::AnalysisId;

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

/// One recorded use. `usage` is absent when the name is only in the used set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedComponentUse {
    pub tag: CompactString,
    pub usage: Option<crate::croquis::ComponentUsage>,
    pub usage_index: Option<u32>,
    pub name_ordinal: Option<u32>,
}

/// The `ComponentUsages` fact group.
pub struct ComponentUsages;

impl FactGroup for ComponentUsages {
    const ID: AnalysisId = ids::COMPONENT_USAGES;
    const NAME: &'static str = "component-usages";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ComponentIdentity;
    type Value = Vec<GroupedComponentUse>;
}

impl FactProducer<Croquis> for ComponentUsages {
    fn produce(croquis: &Croquis, _: &FactView<'_>) -> FactTable<Self> {
        let mut grouped: Vec<(ComponentIdentity, Vec<GroupedComponentUse>)> =
            Vec::with_capacity(croquis.component_usages.len());
        for (index, usage) in croquis.component_usages.iter().enumerate() {
            let identity = resolve(croquis, usage.name.as_str());
            let recorded = GroupedComponentUse {
                tag: usage.name.clone(),
                usage: Some(usage.clone()),
                usage_index: Some(u32::try_from(index).unwrap_or(u32::MAX)),
                name_ordinal: None,
            };
            if let Some((_, sites)) = grouped.iter_mut().find(|(key, _)| key == &identity) {
                sites.push(recorded);
            } else {
                grouped.push((identity, vec![recorded]));
            }
        }
        for (ordinal, name) in croquis.used_components.iter().enumerate() {
            let identity = resolve(croquis, name.as_str());
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            if let Some((_, sites)) = grouped.iter_mut().find(|(key, _)| key == &identity) {
                for site in sites.iter_mut().filter(|site| site.tag == *name) {
                    site.name_ordinal = Some(ordinal);
                }
                if sites.iter().any(|site| site.tag == *name) {
                    continue;
                }
            }
            let recorded = GroupedComponentUse {
                tag: name.clone(),
                usage: None,
                usage_index: None,
                name_ordinal: Some(ordinal),
            };
            if let Some((_, sites)) = grouped.iter_mut().find(|(key, _)| key == &identity) {
                sites.push(recorded);
            } else {
                grouped.push((identity, vec![recorded]));
            }
        }
        grouped.into_iter().collect()
    }
}

/// The identity a template tag renders: through an Options API registration,
/// then through a value import, preferring an exact local name over a
/// Pascal/kebab spelling of it.
pub fn component_identity(croquis: &Croquis, tag: &str) -> ComponentIdentity {
    resolve(croquis, tag)
}

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

/// Source-owned local component binding. This is private CF authority, not a
/// synthetic static import and not part of the serialized component DTO.
pub enum LocalComponentBinding<'a> {
    Absent,
    StaticImport,
    Opaque,
    Async(&'a str),
}

/// A known opaque local must not acquire a project component by filename.
pub fn local_component_binding<'a>(
    croquis: &'a crate::Croquis,
    tag: &str,
) -> LocalComponentBinding<'a> {
    if !croquis.bindings.is_script_setup {
        return LocalComponentBinding::Absent;
    }
    let Some(origins) = croquis
        .value_binding_origins
        .as_deref()
        .filter(|origins| origins.owns_setup_program())
    else {
        return unavailable_binding(croquis, tag);
    };
    let Some(name) = origins.component_binding_name(tag) else {
        return unavailable_binding(croquis, tag);
    };
    let Some(name) = name else {
        return LocalComponentBinding::Opaque;
    };
    if origins.is_syntactic_value_import(name) {
        return LocalComponentBinding::StaticImport;
    }
    if croquis.source_program_count != 1 {
        return LocalComponentBinding::Opaque;
    }
    let Some(reference) = origins.root_reference(name) else {
        return LocalComponentBinding::Opaque;
    };
    let Some(resolved) = origins.resolve(&reference) else {
        return LocalComponentBinding::Opaque;
    };
    if resolved.canonical.is_none() {
        return LocalComponentBinding::Opaque;
    }
    if resolved.import.is_some() {
        // A const alias is not the authored static-import declaration which
        // the legacy component-identity resolver can read. Keep it opaque.
        return LocalComponentBinding::Opaque;
    }
    match origins.async_components.module_for(origins, name) {
        Some(module) => LocalComponentBinding::Async(module),
        None => LocalComponentBinding::Opaque,
    }
}

// Missing capture or another Program's root is not proof that a setup name
// is absent. Reuse the existing declared inventory with bounded hash lookups.
fn metadata_binding_name<'a>(croquis: &'a Croquis, tag: &str) -> Option<&'a str> {
    crate::script_parser::component_binding_reference(tag, |candidate| {
        croquis
            .bindings
            .bindings
            .get_key_value(candidate)
            .map(|(name, _)| name.as_str())
    })
}

fn unavailable_binding<'a>(croquis: &'a Croquis, tag: &str) -> LocalComponentBinding<'a> {
    if croquis.source_program_count == 1 && imported_export(croquis, tag).is_some() {
        // The existing authored static-import route does not depend on Vue
        // factory authority or alias/value provenance from the private store.
        return LocalComponentBinding::StaticImport;
    }
    match metadata_binding_name(croquis, tag) {
        Some(_) => LocalComponentBinding::Opaque,
        None => LocalComponentBinding::Absent,
    }
}

/// The authored local name used when a template spelling selects a setup binding.
/// Scope readers use this identity rather than comparing script/template offsets.
#[doc(hidden)]
pub fn component_reference_name<'a>(croquis: &'a crate::Croquis, tag: &str) -> Option<&'a str> {
    croquis
        .value_binding_origins
        .as_deref()
        .filter(|origins| origins.owns_setup_program())
        .and_then(|origins| origins.component_binding_name(tag))
        .flatten()
        .or_else(|| metadata_binding_name(croquis, tag))
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
    fn uncaptured_setup(source: &str) -> crate::Croquis {
        crate::script_parser::parse_script_setup_with_exports::<false>(
            source, None, false, false, true, false, None,
        )
        .into_croquis()
    }

    #[test]
    fn missing_setup_capture_cannot_reuse_plain_program_names() {
        use super::{LocalComponentBinding, local_component_binding};
        for (source, tag) in [
            ("const Child = () => null;", "Child"),
            ("const Child = () => null;", "child"),
            ("const childWidget = () => null;", "child-widget"),
            ("function Child() { return null; }", "Child"),
        ] {
            for plain in [
                "import { defineAsyncComponent } from 'vue'; const Plain = defineAsyncComponent(() => import('./Plain.vue'));",
                "import Child from './Child.vue';",
            ] {
                let mut setup = uncaptured_setup(source);
                assert!(setup.value_binding_origins.is_none());
                assert_eq!(setup.source_program_count, 1);
                assert!(matches!(
                    local_component_binding(&setup, tag),
                    LocalComponentBinding::Opaque
                ));
                setup.merge_plain_script(crate::script_parser::parse_script(plain).into_croquis());
                assert_eq!(setup.source_program_count, 2);
                assert!(
                    matches!(
                        local_component_binding(&setup, tag),
                        LocalComponentBinding::Opaque
                    ),
                    "{source} {plain} {tag}"
                );

                let mut reversed = crate::script_parser::parse_script(plain).into_croquis();
                reversed.merge_plain_script(uncaptured_setup(source));
                assert_eq!(reversed.source_program_count, 2);
                assert!(
                    matches!(
                        local_component_binding(&reversed, tag),
                        LocalComponentBinding::Opaque
                    ),
                    "reverse {source} {plain} {tag}"
                );
            }
            let mut sole = uncaptured_setup(source);
            sole.merge_plain_script(crate::Croquis::new());
            assert_eq!(sole.source_program_count, 1);
            assert!(matches!(
                local_component_binding(&sole, tag),
                LocalComponentBinding::Opaque
            ));
            assert!(matches!(
                local_component_binding(&sole, "Absent"),
                LocalComponentBinding::Absent
            ));
        }
    }

    #[test]
    fn original_static_import_route_remains_separate_from_capture() {
        use super::{LocalComponentBinding, local_component_binding};
        let source = "import Child from './Child.vue';";
        let uncaptured = uncaptured_setup(source);
        assert!(matches!(
            local_component_binding(&uncaptured, "Child"),
            LocalComponentBinding::StaticImport
        ));
        let mut captured = crate::script_parser::parse_script_setup(source).into_croquis();
        captured.merge_plain_script(
            crate::script_parser::parse_script("const plain = 1;").into_croquis(),
        );
        assert_eq!(captured.source_program_count, 2);
        assert!(matches!(
            local_component_binding(&captured, "Child"),
            LocalComponentBinding::StaticImport
        ));
    }
    #[test]
    fn component_binding_spelling_follows_vue_lookup_order() {
        use super::{LocalComponentBinding, local_component_binding};
        let both = "import { defineAsyncComponent as dac } from 'vue'; const childWidget = dac(() => import('./Camel.vue')); const ChildWidget = dac(() => import('./Pascal.vue'));";
        for (source, tag, expected) in [
            (both, "child-widget", Some("./Camel.vue")),
            (both, "ChildWidget", Some("./Pascal.vue")),
            (both, "childWidget", Some("./Camel.vue")),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const Child_Widget = dac(() => import('./Local.vue'));",
                "ChildWidget",
                None,
            ),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const lower = dac(() => import('./Local.vue'));",
                "Lower",
                None,
            ),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const ChildWidget = dac(() => import('./Pascal.vue'));",
                "child-widget",
                Some("./Pascal.vue"),
            ),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const CaféWidget = dac(() => import('./Pascal.vue'));",
                "café-widget",
                Some("./Pascal.vue"),
            ),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const CafeÉclair = dac(() => import('./Local.vue'));",
                "cafe-éclair",
                None,
            ),
            (
                "import { defineAsyncComponent as dac } from 'vue'; const SSbox = dac(() => import('./Local.vue'));",
                "ßbox",
                None,
            ),
        ] {
            let analysis = crate::script_parser::parse_script_setup(source).into_croquis();
            let actual = match local_component_binding(&analysis, tag) {
                LocalComponentBinding::Async(module) => Some(module),
                _ => None,
            };
            assert_eq!(actual, expected, "{source} {tag}");
        }
    }
}
