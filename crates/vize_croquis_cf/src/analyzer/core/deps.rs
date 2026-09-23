use super::CrossFileAnalyzer;
use crate::facts::resolve_module;
use crate::graph::DependencyEdge;
use crate::registry::FileId;
use std::path::Path;

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
                    Some((data.source.clone(), data.is_type_only))
                } else {
                    None
                }
            })
            .collect();
        self.clear_component_usage_edges(file_id);

        for (source, is_type_only) in imports_data {
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
        }

        for target_id in crate::facts::component_usage_targets(&self.registry, file_id) {
            self.graph
                .add_edge(file_id, target_id, DependencyEdge::ComponentUsage);
        }
    }

    pub(super) fn resolve_import(
        &self,
        specifier: &str,
        from_dir: Option<&Path>,
    ) -> Option<FileId> {
        resolve_module(&self.registry, specifier, from_dir)
    }

    fn clear_component_usage_edges(&mut self, file_id: FileId) {
        let targets = self.graph.get_node(file_id).map_or_else(Vec::new, |node| {
            node.imports
                .iter()
                .filter_map(|(target, edge)| {
                    (*edge == DependencyEdge::ComponentUsage).then_some(*target)
                })
                .collect()
        });

        if let Some(node) = self.graph.get_node_mut(file_id) {
            node.imports
                .retain(|(_, edge)| *edge != DependencyEdge::ComponentUsage);
        }
        for target in targets {
            if let Some(node) = self.graph.get_node_mut(target) {
                node.importers.retain(|(source, edge)| {
                    *source != file_id || *edge != DependencyEdge::ComponentUsage
                });
            }
        }
    }
}

/// Whether an import specifier is relative (`./` or `../`).
///
/// Relative specifiers carry a meaningful directory and must resolve against
/// the importing file's path, so they are excluded from the bare-filename
/// fallback used for flat virtual/playground projects.
fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier == "."
        || specifier == ".."
}

#[cfg(test)]
mod tests {
    use super::is_relative_specifier;
    use crate::{CrossFileAnalyzer, CrossFileOptions, DependencyEdge, FileId};
    use std::path::Path;
    use vize_croquis::Croquis;

    fn analyzer_with(files: &[(&str, &str)]) -> CrossFileAnalyzer {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default());
        for (path, source) in files {
            analyzer.add_file_with_analysis(Path::new(path), source, Croquis::new());
        }
        analyzer
    }

    #[test]
    fn is_relative_specifier_classifies_correctly() {
        assert!(is_relative_specifier("./Button.vue"));
        assert!(is_relative_specifier("../Button.vue"));
        assert!(is_relative_specifier("./nested/Button.vue"));
        assert!(is_relative_specifier("."));
        assert!(is_relative_specifier(".."));
        // Non-relative: bare module, alias, absolute.
        assert!(!is_relative_specifier("Button.vue"));
        assert!(!is_relative_specifier("@/components/Button.vue"));
        assert!(!is_relative_specifier("pinia"));
        assert!(!is_relative_specifier("/abs/Button.vue"));
        // A bare name that merely starts with a dot (extensionless dotfile-ish)
        // is not a relative path because it lacks the `./`/`../` prefix.
        assert!(!is_relative_specifier(".env"));
    }

    /// `./Button.vue` imported from `pages/Home.vue` must NOT resolve to
    /// `admin/Button.vue`: the relative directory is meaningful, so a sibling
    /// that does not exist must stay unresolved rather than fall through to a
    /// same-named file elsewhere via the bare-filename fallback.
    #[test]
    fn relative_import_never_crosses_directories() {
        let analyzer = analyzer_with(&[("pages/Home.vue", ""), ("admin/Button.vue", "")]);

        let from_dir = Path::new("pages/Home.vue").parent();
        assert_eq!(analyzer.resolve_import("./Button.vue", from_dir), None);
    }

    /// The sibling that actually exists is resolved by canonical path.
    #[test]
    fn relative_import_resolves_to_sibling() {
        let analyzer = analyzer_with(&[
            ("pages/Home.vue", ""),
            ("pages/Button.vue", ""),
            ("admin/Button.vue", ""),
        ]);

        let from_dir = Path::new("pages/Home.vue").parent();
        let resolved = analyzer.resolve_import("./Button.vue", from_dir);
        let expected = analyzer.registry().get_id(Path::new("pages/Button.vue"));
        assert!(resolved.is_some());
        assert_eq!(resolved, expected);
    }

    /// Flat in-memory/playground projects have no directory structure, so a
    /// bare (non-relative) specifier still resolves by filename — preserving the
    /// virtual-path scheme the playground relies on.
    #[test]
    fn bare_specifier_resolves_by_filename_for_virtual_projects() {
        let analyzer = analyzer_with(&[("App.vue", ""), ("Button.vue", "")]);

        // No directory information; only the filename is meaningful.
        let resolved = analyzer.resolve_import("Button", None);
        let expected = analyzer.registry().get_id(Path::new("Button.vue"));
        assert!(resolved.is_some());
        assert_eq!(resolved, expected);
    }

    fn import_edge_target(analyzer: &CrossFileAnalyzer, from: FileId, to: FileId) -> bool {
        analyzer
            .graph()
            .nodes()
            .find(|n| n.file_id == from)
            .is_some_and(|node| {
                node.imports
                    .iter()
                    .any(|(target, edge)| *target == to && matches!(edge, DependencyEdge::Import))
            })
    }

    /// End-to-end through the dependency graph and the real single-file import
    /// parser: a relative `import` whose sibling does not exist must not create
    /// an `Import` edge to a same-named file in a different directory.
    #[test]
    fn relative_import_edge_does_not_cross_directories() {
        // `pages/Home.ts` imports `./Button.vue`, but the only `Button.vue`
        // lives in `admin/`.
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default());
        let home_id = analyzer.add_file(
            Path::new("pages/Home.ts"),
            "import Button from './Button.vue'\n",
        );
        analyzer.add_file(Path::new("admin/Button.vue"), "");
        analyzer.rebuild_import_edges();

        let admin_id = analyzer
            .registry()
            .get_id(Path::new("admin/Button.vue"))
            .unwrap();

        assert!(
            !import_edge_target(&analyzer, home_id, admin_id),
            "relative import must not create an Import edge across directories"
        );
    }

    /// The Import edge IS created when the relative sibling exists.
    #[test]
    fn relative_import_edge_resolves_to_sibling() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default());
        let home_id = analyzer.add_file(
            Path::new("pages/Home.ts"),
            "import Button from './Button.vue'\n",
        );
        analyzer.add_file(Path::new("pages/Button.vue"), "");
        analyzer.add_file(Path::new("admin/Button.vue"), "");
        analyzer.rebuild_import_edges();

        let sibling_id = analyzer
            .registry()
            .get_id(Path::new("pages/Button.vue"))
            .unwrap();
        let admin_id = analyzer
            .registry()
            .get_id(Path::new("admin/Button.vue"))
            .unwrap();

        assert!(
            import_edge_target(&analyzer, home_id, sibling_id),
            "relative import must edge to the sibling"
        );
        assert!(
            !import_edge_target(&analyzer, home_id, admin_id),
            "relative import must not also edge to the unrelated admin/Button.vue"
        );
    }

    #[test]
    fn runtime_extension_import_resolves_to_authored_source() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default());
        let consumer_id = analyzer.add_file(
            Path::new("src/consumer.ts"),
            "import { value } from './state.js'\nvoid value\n",
        );
        let source_id = analyzer.add_file(Path::new("src/state.ts"), "export const value = 1\n");

        analyzer.rebuild_import_edges();

        assert!(
            import_edge_target(&analyzer, consumer_id, source_id),
            "runtime extension should resolve to the authored source file"
        );
    }
}
