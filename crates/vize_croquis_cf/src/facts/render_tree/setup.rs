//! Read Doctor's render sites and the dependency graph's ordered targets from one import query.
use super::{
    LocalComponentBinding, RenderEdgeKey, RenderSite, component_identity, follow_forwards,
    local_component_binding, names_match, resolve_module, template_usage_is_shadowed,
    unique_component,
};
use crate::{FileId, ModuleRegistry};
use std::{collections::BTreeMap, path::Path};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::facts::{component_usage_slice, used_component_name_list, used_components_empty};
use vize_l1_to_l2::native_file::SetupComponentImports;

/// Native tag and import facts paired with the revision stored by the project registry.
pub struct SetupRenderInput<'q, 'o, 'a> {
    imports: &'q SetupComponentImports<'o, 'a>,
    revision: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SetupRenderProducts {
    pub edges: BTreeMap<RenderEdgeKey, Vec<RenderSite>>,
    pub targets: Vec<FileId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupRenderRefusal {
    Caller,
    SourceRevision,
    UsageCoverage,
    ImportIdentity,
}

impl ModuleRegistry {
    /// Register a Vue caller with its original imports. Carry the registry's
    /// existing revision into later reads, including callers with empty templates.
    pub fn register_setup_caller<'q, 'o, 'a>(
        &mut self,
        path: &Path,
        analysis: vize_croquis::Croquis,
        imports: &'q SetupComponentImports<'o, 'a>,
    ) -> Result<(FileId, SetupRenderInput<'q, 'o, 'a>), SetupRenderRefusal> {
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vue"))
        {
            return Err(SetupRenderRefusal::Caller);
        }
        let (id, _) = self.register(path, imports.source(), analysis);
        let entry = self.get(id).ok_or(SetupRenderRefusal::Caller)?;
        Ok((
            id,
            SetupRenderInput {
                imports,
                revision: entry.source_hash,
            },
        ))
    }

    /// Resolve the registered caller's original component sites against the current
    /// project, retaining both the render table's ranges and graph traversal order.
    pub fn setup_render_products(
        &self,
        path: &Path,
        input: &SetupRenderInput<'_, '_, '_>,
    ) -> Result<SetupRenderProducts, SetupRenderRefusal> {
        let entry = self
            .get_by_path(path)
            .filter(|entry| entry.is_vue_sfc)
            .ok_or(SetupRenderRefusal::Caller)?;
        if entry.source_hash != input.revision {
            return Err(SetupRenderRefusal::SourceRevision);
        }
        let imports = input.imports;
        if imports.sites().is_empty() && used_components_empty(&entry.analysis) {
            return Ok(SetupRenderProducts::default());
        }
        let usages = component_usage_slice(&entry.analysis);
        if usages.len() != imports.sites().len() {
            return Err(SetupRenderRefusal::UsageCoverage);
        }
        let mut edges = BTreeMap::<_, Vec<RenderSite>>::new();
        let mut targets = Vec::new();
        let mut seen_targets = FxHashSet::default();
        let mut seen_sites = FxHashSet::default();
        let mut covered = FxHashSet::default();
        let mut resolutions = FxHashMap::default();
        let template = imports.template_span();
        for usage in usages {
            let mismatch = SetupRenderRefusal::UsageCoverage;
            let start = template.start.checked_add(usage.start).ok_or(mismatch)?;
            let end = template.start.checked_add(usage.end).ok_or(mismatch)?;
            let site = start
                .checked_add(1)
                .and_then(|name| imports.at_offset(name))
                .ok_or(mismatch)?;
            // The registry can store only the opening tag for a paired element.
            // Keep its published range while checking the complete native owner.
            if site.element_span().start != start
                || end <= site.name_span().end
                || end > site.element_span().end
                || imports.source().as_bytes().get((end - 1) as usize) != Some(&b'>')
                || !names_match(site.tag(), usage.name.as_str())
                || !seen_sites.insert(site.node())
            {
                return Err(mismatch);
            }
            covered.insert(usage.name.as_str());
            if template_usage_is_shadowed(&entry.analysis, usage) {
                continue;
            }
            let key = (site.binding().id(), usage.name.as_str());
            let resolved = resolutions.entry(key).or_insert_with(|| {
                let identity = component_identity(&entry.analysis, usage.name.as_str());
                if !matches!(
                    local_component_binding(&entry.analysis, usage.name.as_str()),
                    LocalComponentBinding::StaticImport | LocalComponentBinding::Absent
                ) || identity.module.as_deref() != Some(site.module())
                    || identity.export_name.as_str() != site.export()
                {
                    return Err(SetupRenderRefusal::ImportIdentity);
                }
                Ok(resolve_module(self, site.module(), entry.path.parent())
                    .and_then(|id| follow_forwards(self, id, CompactString::new(site.export()), 0))
                    .or_else(|| {
                        unique_component(self, usage.name.as_str())
                            .map(|id| (id, CompactString::new("default")))
                    }))
            });
            if let Some((target, export)) = resolved.as_ref().map_err(|reason| *reason)? {
                edges
                    .entry(RenderEdgeKey {
                        caller: entry.id,
                        target: *target,
                        export_name: export.clone(),
                    })
                    .or_default()
                    .push(RenderSite {
                        start: usage.start,
                        end: usage.end,
                    });
                if seen_targets.insert(*target) {
                    targets.push(*target);
                }
            }
        }
        // Some registrations know a component without retaining its template use.
        // These names carry independent graph edges and cannot be silently omitted.
        if used_component_name_list(&entry.analysis)
            .iter()
            .any(|name| {
                !covered.contains(name.as_str())
                    && !covered.iter().any(|tag| names_match(tag, name.as_str()))
            })
        {
            return Err(SetupRenderRefusal::UsageCoverage);
        }
        for sites in edges.values_mut() {
            sites.sort_unstable_by_key(|site| (site.start, site.end));
        }
        Ok(SetupRenderProducts { edges, targets })
    }
}

#[cfg(test)]
mod tests;
