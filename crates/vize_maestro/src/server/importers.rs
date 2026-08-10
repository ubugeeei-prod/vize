//! Reverse dependency index for open Vue documents.
mod dependents;
mod index;
mod package;

use std::path::{Component, Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::Url;
use vize_carton::FxHashSet;

use self::package::resolve_package_import;
use super::ServerState;
pub(super) use dependents::open_vue_dependents;
pub(in crate::server) use index::OpenVueImportIndex;

const SCRIPT_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

pub(super) fn open_vue_importers(state: &ServerState, dependency: &Url) -> Vec<Url> {
    dependency
        .to_file_path()
        .ok()
        .map(|path| state.open_vue_imports.importers(&path))
        .unwrap_or_default()
}

#[cfg(any(test, feature = "native"))]
pub(super) fn indexed_dependency_paths(state: &ServerState, dependency: &Path) -> Vec<PathBuf> {
    state.open_vue_imports.dependency_paths(dependency)
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
    let importer_uri = Url::from_file_path(importer).ok();
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
            importer_uri.as_ref(),
            &mut dependencies,
        );
    }
    dependencies.into_iter().collect()
}

fn collect_script_dependencies(
    source: &str,
    source_type: SourceType,
    importer_dir: &Path,
    importer_uri: Option<&Url>,
    dependencies: &mut FxHashSet<PathBuf>,
) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let specifier = import.source.value.as_str();
        if let Some(dependency) = resolve_import(importer_dir, specifier).or_else(|| {
            crate::ide::definition::import_resolver::resolve_import_specifier(
                importer_uri?,
                specifier,
            )
        }) {
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
mod tests;
