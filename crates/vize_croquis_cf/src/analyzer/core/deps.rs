use super::CrossFileAnalyzer;
use super::paths::{component_names_match, import_candidates};
use crate::graph::DependencyEdge;
use crate::registry::FileId;
use std::path::Path;
use vize_carton::{CompactString, String};

impl CrossFileAnalyzer {
    pub(super) fn update_dependency_edges(&mut self, file_id: FileId) {
        let Some(entry) = self.registry.get(file_id) else {
            return;
        };

        let current_dir = entry.path.parent().map(Path::to_path_buf);
        let imports_data: Vec<_> = entry
            .analysis
            .scopes
            .iter()
            .filter_map(|scope| {
                if let vize_croquis::ScopeData::ExternalModule(data) = scope.data() {
                    Some((
                        data.source.clone(),
                        data.is_type_only,
                        scope
                            .bindings()
                            .map(|(name, _)| CompactString::new(name))
                            .collect::<Vec<_>>(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        let used_components: Vec<_> = entry.analysis.used_components.iter().cloned().collect();

        let mut import_bindings = Vec::new();
        for (source, is_type_only, local_bindings) in imports_data {
            let Some(target_id) = self.resolve_import(source.as_str(), current_dir.as_deref())
            else {
                continue;
            };

            let edge_type = if is_type_only {
                DependencyEdge::TypeImport
            } else {
                DependencyEdge::Import
            };
            self.graph.add_edge(file_id, target_id, edge_type);

            if !is_type_only {
                for local in local_bindings {
                    import_bindings.push((local, target_id));
                }
            }
        }

        for component in used_components {
            let target_id = import_bindings
                .iter()
                .find(|(local, _)| component_names_match(component.as_str(), local.as_str()))
                .map(|(_, target_id)| *target_id)
                .or_else(|| self.find_component_by_name(component.as_str()));

            if let Some(target_id) = target_id {
                self.graph
                    .add_edge(file_id, target_id, DependencyEdge::ComponentUsage);
            }
        }
    }

    pub(super) fn resolve_import(
        &self,
        specifier: &str,
        from_dir: Option<&Path>,
    ) -> Option<FileId> {
        for candidate in import_candidates(specifier, from_dir) {
            if let Some(entry) = self.registry.get_by_path(&candidate) {
                return Some(entry.id);
            }
        }

        // Fallback for flat in-memory playground projects and non-relative
        // imports where only the filename is meaningful.
        let basename = specifier
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or(specifier);

        let mut vue_basename = String::from(basename);
        vue_basename.push_str(".vue");
        let mut ts_basename = String::from(basename);
        ts_basename.push_str(".ts");

        for entry in self.registry.iter() {
            if entry.filename.as_str() == basename
                || entry.filename.as_str() == vue_basename
                || entry.filename.as_str() == ts_basename
            {
                return Some(entry.id);
            }
        }

        None
    }

    pub(super) fn find_component_by_name(&self, name: &str) -> Option<FileId> {
        self.graph.find_by_component(name)
    }
}
