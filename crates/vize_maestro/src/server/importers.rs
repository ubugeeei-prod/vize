//! Fast discovery of open Vue files that directly import a changed file.

use std::path::{Component, Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::Url;

use super::ServerState;

pub(super) fn open_vue_importers(state: &ServerState, dependency: &Url) -> Vec<Url> {
    let Ok(dependency_path) = dependency.to_file_path() else {
        return Vec::new();
    };
    let dependency_path = comparable_path(&dependency_path);

    state
        .documents
        .iter()
        .filter_map(|document| {
            let uri = document.key();
            if uri == dependency || !uri.path().ends_with(".vue") {
                return None;
            }

            let importer_path = uri.to_file_path().ok()?;
            imports_path(&importer_path, &document.value().text(), &dependency_path)
                .then(|| uri.clone())
        })
        .collect()
}

fn imports_path(importer: &Path, source: &str, dependency: &Path) -> bool {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: importer.to_string_lossy().into_owned().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
        return false;
    };
    let Some(importer_dir) = importer.parent() else {
        return false;
    };

    descriptor
        .script
        .iter()
        .chain(descriptor.script_setup.iter())
        .any(|script| {
            script_imports_path(
                script.content.as_ref(),
                source_type(script.lang.as_deref()),
                importer_dir,
                dependency,
            )
        })
}

fn script_imports_path(
    source: &str,
    source_type: SourceType,
    importer_dir: &Path,
    dependency: &Path,
) -> bool {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    parsed.program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        resolves_to(importer_dir, import.source.value.as_str(), dependency)
    })
}

fn resolves_to(importer_dir: &Path, specifier: &str, dependency: &Path) -> bool {
    let specifier = specifier
        .split_once(['?', '#'])
        .map_or(specifier, |(path, _)| path);
    if !specifier.starts_with("./") && !specifier.starts_with("../") {
        return false;
    }

    let joined = importer_dir.join(specifier);
    let candidates = if Path::new(specifier).extension().is_some() {
        vec![joined]
    } else {
        vec![joined.with_extension("vue"), joined.join("index.vue")]
    };
    candidates
        .iter()
        .any(|candidate| comparable_path(candidate) == dependency)
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

    use super::open_vue_importers;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn finds_only_open_vue_files_that_import_changed_component() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("Child.vue");
        let parent = dir.path().join("Parent.vue");
        let unrelated = dir.path().join("Unrelated.vue");
        std::fs::write(&child, "<template />").unwrap();
        std::fs::write(&parent, "<template />").unwrap();
        std::fs::write(&unrelated, "<template />").unwrap();

        let child_uri = Url::from_file_path(&child).unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let unrelated_uri = Url::from_file_path(&unrelated).unwrap();
        let state = ServerState::new();
        state.documents.open(
            child_uri.clone(),
            "<template />".to_string(),
            2,
            "vue".to_string(),
        );
        state.documents.open(
            parent_uri.clone(),
            "<script setup lang=\"ts\">\nimport Child from './Child'\n</script>".to_string(),
            1,
            "vue".to_string(),
        );
        state.documents.open(
            unrelated_uri,
            "<script>import Other from './Other.vue'</script>".to_string(),
            1,
            "vue".to_string(),
        );

        assert_eq!(open_vue_importers(&state, &child_uri), vec![parent_uri]);
    }

    #[test]
    fn detects_imports_from_normal_script_with_query_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("Child.vue");
        let parent = dir.path().join("Parent.vue");
        std::fs::write(&child, "<template />").unwrap();
        std::fs::write(&parent, "<template />").unwrap();

        let child_uri = Url::from_file_path(&child).unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let state = ServerState::new();
        state.documents.open(
            parent_uri.clone(),
            "<script>import Child from './Child.vue?vue&type=script'</script>".to_string(),
            1,
            "vue".to_string(),
        );

        assert_eq!(open_vue_importers(&state, &child_uri), vec![parent_uri]);
    }
}
