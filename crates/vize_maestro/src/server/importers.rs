//! Reverse dependency index for open Vue documents.
mod dependents;
use std::path::{Component, Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_carton::{FxHashMap, FxHashSet};

use self::package::resolve_package_import;
use super::ServerState;
pub(super) use dependents::open_vue_dependents;
mod package;

const SCRIPT_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

#[derive(Default)]
pub(super) struct OpenVueImportIndex {
    inner: RwLock<ImportIndexData>,
}

#[derive(Default)]
struct ImportIndexData {
    by_dependency: FxHashMap<PathBuf, FxHashSet<Url>>,
    by_importer: FxHashMap<Url, Vec<PathBuf>>,
}

impl OpenVueImportIndex {
    pub(super) fn update(&self, importer: &Url, source: &str) {
        let dependencies = importer
            .to_file_path()
            .ok()
            .filter(|path| path.extension().is_some_and(|extension| extension == "vue"))
            .map(|path| collect_dependencies(&path, source))
            .unwrap_or_default();
        let mut index = self.inner.write();
        remove_importer(&mut index, importer);

        for dependency in &dependencies {
            index
                .by_dependency
                .entry(dependency.clone())
                .or_default()
                .insert(importer.clone());
        }
        if !dependencies.is_empty() {
            index.by_importer.insert(importer.clone(), dependencies);
        }
    }

    pub(super) fn remove(&self, importer: &Url) {
        remove_importer(&mut self.inner.write(), importer);
    }

    pub(super) fn clear(&self) {
        let mut index = self.inner.write();
        index.by_dependency.clear();
        index.by_importer.clear();
    }

    fn importers(&self, dependency: &Path) -> Vec<Url> {
        self.inner
            .read()
            .by_dependency
            .get(&comparable_path(dependency))
            .map(|importers| importers.iter().cloned().collect())
            .unwrap_or_default()
    }
}

fn remove_importer(index: &mut ImportIndexData, importer: &Url) {
    let Some(dependencies) = index.by_importer.remove(importer) else {
        return;
    };
    for dependency in dependencies {
        if let Some(importers) = index.by_dependency.get_mut(&dependency) {
            importers.remove(importer);
            if importers.is_empty() {
                index.by_dependency.remove(&dependency);
            }
        }
    }
}

pub(super) fn open_vue_importers(state: &ServerState, dependency: &Url) -> Vec<Url> {
    dependency
        .to_file_path()
        .ok()
        .map(|path| state.open_vue_imports.importers(&path))
        .unwrap_or_default()
}

fn collect_dependencies(importer: &Path, source: &str) -> Vec<PathBuf> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: importer.to_string_lossy().into_owned().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
        return Vec::new();
    };
    let Some(importer_dir) = importer.parent() else {
        return Vec::new();
    };
    let mut dependencies = FxHashSet::default();

    for script in descriptor
        .script
        .iter()
        .chain(descriptor.script_setup.iter())
    {
        collect_script_dependencies(
            script.content.as_ref(),
            source_type(script.lang.as_deref()),
            importer_dir,
            &mut dependencies,
        );
    }
    dependencies.into_iter().collect()
}

fn collect_script_dependencies(
    source: &str,
    source_type: SourceType,
    importer_dir: &Path,
    dependencies: &mut FxHashSet<PathBuf>,
) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if let Some(dependency) = resolve_import(importer_dir, import.source.value.as_str()) {
            dependencies.insert(dependency);
        }
    }
}

fn resolve_import(importer_dir: &Path, specifier: &str) -> Option<PathBuf> {
    let specifier = specifier
        .split_once(['?', '#'])
        .map_or(specifier, |(path, _)| path);
    if specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
    {
        return resolve_relative_import(importer_dir, specifier);
    }

    resolve_package_import(importer_dir, specifier)
}

fn resolve_relative_import(importer_dir: &Path, specifier: &str) -> Option<PathBuf> {
    let joined = importer_dir.join(specifier);
    if matches!(specifier, "." | "..") {
        return SCRIPT_EXTENSIONS
            .iter()
            .map(|extension| joined.join("index").with_extension(extension))
            .find(|candidate| candidate.exists())
            .map(|candidate| comparable_path(&candidate));
    }
    if Path::new(specifier).extension().is_some() {
        return Some(comparable_path(&joined));
    }
    SCRIPT_EXTENSIONS
        .iter()
        .map(|extension| joined.with_extension(extension))
        .chain(
            SCRIPT_EXTENSIONS
                .iter()
                .map(|extension| joined.join("index").with_extension(extension)),
        )
        .find(|candidate| candidate.exists())
        .map(|candidate| comparable_path(&candidate))
}

fn comparable_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                _ => normalized.push(component.as_os_str()),
            }
        }
        normalized
    })
}

fn source_type(lang: Option<&str>) -> SourceType {
    match lang.unwrap_or("js") {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]

    use super::{open_vue_importers, resolve_import};
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn index_tracks_and_removes_open_vue_imports() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("Child.vue");
        let parent = dir.path().join("Parent.vue");
        std::fs::write(&child, "<template />").unwrap();
        std::fs::write(&parent, "<template />").unwrap();
        let child_uri = Url::from_file_path(&child).unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let state = ServerState::new();
        let source = "<script setup lang=\"ts\">import Child from './Child'</script>";

        state.update_virtual_docs(&parent_uri, source);
        assert_eq!(
            open_vue_importers(&state, &child_uri),
            vec![parent_uri.clone()]
        );

        state.update_virtual_docs(&parent_uri, "<script setup>const local = 1</script>");
        assert!(open_vue_importers(&state, &child_uri).is_empty());
    }

    #[test]
    fn index_resolves_explicit_script_dependencies_and_query_suffixes() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("types.ts");
        let parent = dir.path().join("Parent.vue");
        std::fs::write(&child, "export type Count = number").unwrap();
        std::fs::write(&parent, "<template />").unwrap();
        let child_uri = Url::from_file_path(&child).unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let state = ServerState::new();
        let source = "<script>import './types.ts?raw'</script>";

        state.update_virtual_docs(&parent_uri, source);
        assert_eq!(open_vue_importers(&state, &child_uri), vec![parent_uri]);
    }

    #[test]
    fn index_resolves_package_export_declaration_variants() {
        let dir = tempfile::tempdir().unwrap();
        let package = dir.path().join("node_modules/vue-router");
        let parent = dir.path().join("Parent.vue");
        let module_declaration = package.join("routes.d.mts");
        let common_declaration = package.join("plugin.d.cts");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{
  "exports": {
    "./auto-routes": { "types": "./routes.d.mts" },
    "./volar/plugin": { "types": "./plugin.d.cts" }
  }
}"#,
        )
        .unwrap();
        std::fs::write(
            &module_declaration,
            "export declare const routes: unknown[]",
        )
        .unwrap();
        std::fs::write(&common_declaration, "export declare const plugin: unknown").unwrap();
        std::fs::write(&parent, "<template />").unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let module_uri = Url::from_file_path(&module_declaration).unwrap();
        let common_uri = Url::from_file_path(&common_declaration).unwrap();
        let state = ServerState::new();
        let source = r#"<script setup lang="ts">
import { routes } from 'vue-router/auto-routes'
import { plugin } from 'vue-router/volar/plugin'
void routes
void plugin
</script>"#;

        state.update_virtual_docs(&parent_uri, source);

        assert_eq!(
            open_vue_importers(&state, &module_uri),
            vec![parent_uri.clone()]
        );
        assert_eq!(open_vue_importers(&state, &common_uri), vec![parent_uri]);
    }

    #[test]
    fn exact_directory_specifiers_resolve_index_files() {
        let dir = tempfile::tempdir().unwrap();
        let source_dir = dir.path().join("src");
        let source_index = source_dir.join("index.ts");
        let parent_index = dir.path().join("index.ts");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(&source_index, "export const source = true").unwrap();
        std::fs::write(&parent_index, "export const parent = true").unwrap();
        std::fs::write(dir.path().join("src.vue"), "<template />").unwrap();

        assert_eq!(
            resolve_import(&source_dir, ".?raw"),
            Some(std::fs::canonicalize(&source_index).unwrap())
        );
        assert_eq!(
            resolve_import(&source_dir, "..#parent"),
            Some(std::fs::canonicalize(&parent_index).unwrap())
        );
    }
}
