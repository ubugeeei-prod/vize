//! Conservative Nuxt domains from already collected facts and graph edges.
//! This never parses source, resolves files, or changes the dependency graph.

use super::IdLocation;
use crate::NuxtRenderDirs;
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use std::path::{Path, PathBuf};
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::ScopeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Domain {
    Page,
    Layout,
}

pub(super) struct NuxtIdDomains<'a> {
    alternatives: FxHashMap<FileId, (Domain, &'a Path)>,
}

impl<'a> NuxtIdDomains<'a> {
    pub(super) fn new(
        registry: &'a ModuleRegistry,
        graph: &DependencyGraph,
        configured: Option<&NuxtRenderDirs>,
        complete_scope: bool,
    ) -> Option<Self> {
        if !complete_scope {
            return None;
        }
        let configured = configured?;
        if !configured.pages_enabled || !configured.static_render_config {
            return None;
        }
        let resolve = |path: &Path| -> PathBuf {
            match registry.project_root() {
                Some(root) if path.is_relative() => root.join(path),
                _ => path.to_path_buf(),
            }
        };
        let src = resolve(&configured.src);
        let pages = resolve(&configured.pages);
        let layouts = resolve(&configured.layouts);
        if pages.starts_with(&layouts)
            || layouts.starts_with(&pages)
            || pages == src
            || layouts == src
        {
            return None;
        }
        let mut apps = registry.vue_components().filter(|entry| {
            entry.path.parent() == Some(src.as_path())
                && matches!(
                    entry.path.file_name().and_then(|name| name.to_str()),
                    Some("app.vue" | "App.vue")
                )
        });
        let app = apps.next()?;
        if apps.next().is_some()
            || graph
                .dependents(app.id)
                .any(|(_, edge)| runtime_import(edge))
        {
            return None;
        }

        let mut alternatives = FxHashMap::default();
        let mut page_outlets = 0_u32;
        let mut layout_outlets = 0_u32;
        for entry in registry.vue_components() {
            let role = if let Ok(relative) = entry.path.strip_prefix(&pages) {
                Some((Domain::Page, page_alternative(relative)?))
            } else if let Ok(relative) = entry.path.strip_prefix(&layouts) {
                Some((Domain::Layout, relative))
            } else {
                None
            };
            if entry.id != app.id && role.is_none() {
                continue;
            }
            // No unclassified script, recovered template, runtime expression, or
            // custom component can establish this intentionally closed proof.
            let template = entry.analysis.nuxt_render_template?;
            let expected_slots = u32::from(matches!(role, Some((Domain::Layout, _))));
            if entry.analysis.nuxt_static_render_script != Some(true)
                || !template.admitted
                || template.opaque_structure
                || template.slot_outlets != expected_slots
                || !entry.analysis.template_expressions.is_empty()
                || !entry.analysis.used_directives.is_empty()
            {
                return None;
            }
            for usage in &entry.analysis.component_usages {
                if entry.id != app.id
                    || usage.scope_id != ScopeId::ROOT
                    || usage.vif_guard.is_some()
                    || usage.has_spread_attrs
                    || !usage.spread_props.is_empty()
                    || !usage.events.is_empty()
                {
                    return None;
                }
                match usage.name.as_str() {
                    "NuxtPage" | "nuxt-page"
                        if usage.props.is_empty() && usage.slots.is_empty() =>
                    {
                        page_outlets += 1;
                    }
                    "NuxtLayout" | "nuxt-layout"
                        if usage.slots.len() <= 1
                            && usage.slots.iter().all(|slot| {
                                slot.name == "default"
                                    && !slot.name_is_dynamic
                                    && !slot.has_scope
                                    && slot.scope_vars.is_empty()
                            })
                            && usage.props.iter().all(|prop| {
                                prop.name == "name"
                                    && !prop.is_dynamic
                                    && !prop.name_is_dynamic
                                    && prop.value.is_some()
                            }) =>
                    {
                        layout_outlets += 1;
                    }
                    _ => return None,
                }
            }
            if let Some(domain) = role
                && !graph
                    .dependents(entry.id)
                    .any(|(_, edge)| runtime_import(edge))
            {
                alternatives.insert(entry.id, domain);
            }
        }
        if page_outlets > 1 || layout_outlets > 1 || (page_outlets == 0 && layout_outlets == 0) {
            return None;
        }
        alternatives.retain(|_, (domain, _)| match domain {
            Domain::Page => page_outlets == 1,
            Domain::Layout => layout_outlets == 1,
        });
        Some(Self { alternatives })
    }

    /// A group is excluded only when every location proves the same outlet
    /// domain and a different alternative. One unknown or co-rendering location
    /// retains the original whole group. No all-location-pairs search is needed.
    pub(super) fn all_exclusive(&self, locations: &[IdLocation]) -> bool {
        let mut domain = None;
        let mut alternatives = FxHashSet::default();
        for location in locations {
            if location.in_loop {
                return false;
            }
            let Some(&(next_domain, alternative)) = self.alternatives.get(&location.file_id) else {
                return false;
            };
            if domain.is_some_and(|current| current != next_domain)
                || !alternatives.insert(alternative)
            {
                return false;
            }
            domain = Some(next_domain);
        }
        locations.len() > 1
    }
}

fn runtime_import(edge: DependencyEdge) -> bool {
    matches!(
        edge,
        DependencyEdge::Import | DependencyEdge::DynamicImport | DependencyEdge::ReExport
    )
}

fn page_alternative(relative: &Path) -> Option<&Path> {
    if relative
        .parent()
        .is_some_and(|parent| parent.as_os_str().is_empty())
    {
        return relative.file_stem().map(Path::new);
    }
    relative.iter().next().map(Path::new)
}
