//! Discovery of project declarations that augment Vue global components.

use std::path::{Path, PathBuf};

use ignore::{DirEntry, WalkBuilder};

use super::ServerState;

impl ServerState {
    pub(crate) fn global_component_reference_paths(&self) -> Vec<PathBuf> {
        if let Some(paths) = self.global_component_reference_paths.read().as_ref() {
            return paths.clone();
        }

        let paths = self.get_workspace_root().map_or_else(Vec::new, |root| {
            collect_global_component_declarations(&root)
        });
        *self.global_component_reference_paths.write() = Some(paths.clone());
        paths
    }
}

fn collect_global_component_declarations(root: &Path) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .ignore(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .parents(false)
        .follow_links(false)
        .filter_entry(should_visit);

    let mut paths = Vec::new();
    for entry in builder.build().flatten() {
        let path = entry.path();
        if !is_declaration_file(path) {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() > 4 * 1024 * 1024 {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if content.contains("GlobalComponents") && content.contains("declare module") {
            paths.push(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()));
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn should_visit(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_some_and(|kind| kind.is_dir()) {
        return true;
    }
    !matches!(
        entry.file_name().to_str(),
        Some(".git" | "node_modules" | "target" | "coverage" | "dist")
    )
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

#[cfg(test)]
mod tests {
    use super::collect_global_component_declarations;

    #[test]
    fn finds_hidden_project_augmentations_without_scanning_dependencies() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".nuxt")).unwrap();
        std::fs::create_dir_all(root.path().join("node_modules/pkg")).unwrap();
        std::fs::write(
            root.path().join("components.d.ts"),
            "declare module 'vue' { interface GlobalComponents { RootCard: unknown } }",
        )
        .unwrap();
        std::fs::write(
            root.path().join(".nuxt/components.d.ts"),
            "declare module 'vue' { interface GlobalComponents { NuxtCard: unknown } }",
        )
        .unwrap();
        std::fs::write(
            root.path().join("node_modules/pkg/global.d.ts"),
            "declare module 'vue' { interface GlobalComponents { DependencyCard: unknown } }",
        )
        .unwrap();
        std::fs::write(root.path().join("unrelated.d.ts"), "interface Plain {}\n").unwrap();

        let paths = collect_global_component_declarations(root.path());
        assert_eq!(paths.len(), 2, "{paths:?}");
        assert!(paths.iter().any(|path| path.ends_with("components.d.ts")));
        assert!(
            paths
                .iter()
                .any(|path| path.ends_with(".nuxt/components.d.ts"))
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.starts_with(root.path().join("node_modules")))
        );
    }
}
