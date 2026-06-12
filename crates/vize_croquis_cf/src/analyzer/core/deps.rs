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

        // Bare-filename fallback for flat in-memory/playground projects where
        // files have no real directory structure (a "virtual path" scheme):
        // every file lives at the root, so only the filename is meaningful and
        // a bare specifier like `Button` is matched to `Button.vue`.
        //
        // This fallback must NOT fire for relative specifiers (`./`, `../`):
        // their directory is meaningful, so `./Button.vue` may only resolve to
        // a sibling, never to a same-named file in a different directory (e.g.
        // `admin/Button.vue`). Restricting the fallback to non-relative
        // specifiers keeps the virtual-path case working while preventing
        // cross-directory leakage.
        if is_relative_specifier(specifier) {
            return None;
        }

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
}
