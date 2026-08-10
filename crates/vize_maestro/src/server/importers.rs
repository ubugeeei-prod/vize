//! Reverse dependency index for open SFC and script documents.

mod dependents;
mod package;
mod specifiers;

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use oxc_span::SourceType;
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_carton::{FxHashMap, FxHashSet};

use self::package::resolve_package_import;
use super::ServerState;
pub(super) use dependents::open_typecheck_dependents;

const SCRIPT_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

#[derive(Default)]
pub(super) struct OpenImportIndex {
    inner: RwLock<ImportIndexData>,
}

#[derive(Default)]
struct ImportIndexData {
    by_dependency: BTreeMap<PathBuf, FxHashSet<Url>>,
    by_importer: FxHashMap<Url, Vec<PathBuf>>,
}

impl OpenImportIndex {
    pub(super) fn update(&self, importer: &Url, source: &str) {
        let dependencies = importer
            .to_file_path()
            .ok()
            .map(|path| collect_dependencies(&path, importer, source))
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
        let dependency = comparable_path(dependency);
        let index = self.inner.read();
        let mut importers = index
            .by_dependency
            .range(dependency.clone()..)
            .take_while(|(path, _)| path.starts_with(&dependency))
            .flat_map(|(_, importers)| importers.iter().cloned())
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        importers.sort();
        importers
    }

    #[cfg(any(test, feature = "native"))]
    fn dependency_paths(&self, dependency: &Path) -> Vec<PathBuf> {
        let dependency = comparable_path(dependency);
        let index = self.inner.read();
        index
            .by_dependency
            .range(dependency.clone()..)
            .take_while(|(path, _)| path.starts_with(&dependency))
            .map(|(path, _)| path.clone())
            .collect()
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

pub(super) fn open_importers(state: &ServerState, dependency: &Url) -> Vec<Url> {
    dependency
        .to_file_path()
        .ok()
        .map(|path| state.open_imports.importers(&path))
        .unwrap_or_default()
}

#[cfg(any(test, feature = "native"))]
pub(super) fn indexed_dependency_paths(state: &ServerState, dependency: &Path) -> Vec<PathBuf> {
    state.open_imports.dependency_paths(dependency)
}

impl ServerState {
    #[cfg(any(test, feature = "native"))]
    pub(crate) fn open_importers(&self, dependency: &Url) -> Vec<Url> {
        open_importers(self, dependency)
    }
}

fn collect_dependencies(importer: &Path, importer_uri: &Url, source: &str) -> Vec<PathBuf> {
    let Some(importer_dir) = importer.parent() else {
        return Vec::new();
    };
    if importer
        .extension()
        .is_none_or(|extension| extension != "vue")
    {
        let Ok(source_type) = SourceType::from_path(importer) else {
            return Vec::new();
        };
        let mut dependencies = FxHashSet::default();
        collect_script_dependencies(
            source,
            source_type,
            importer_dir,
            Some(importer_uri),
            &mut dependencies,
        );
        return dependencies.into_iter().collect();
    }
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: importer.to_string_lossy().into_owned().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
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
            Some(importer_uri),
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
    for specifier in specifiers::collect(source, source_type) {
        let specifier = specifier.as_str();
        if let Some(dependency) = resolve_import(importer_dir, specifier)
            .or_else(|| {
                crate::ide::definition::import_resolver::resolve_import_specifier(
                    importer_uri?,
                    specifier,
                )
            })
            .map(|dependency| comparable_path(&dependency))
        {
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
    let mut ancestor = path;
    let mut unresolved = Vec::new();
    while let Some(parent) = ancestor.parent() {
        if let Some(name) = ancestor.file_name() {
            unresolved.push(name.to_os_string());
        }
        if let Ok(mut canonical) = std::fs::canonicalize(parent) {
            for component in unresolved.into_iter().rev() {
                canonical.push(component);
            }
            return canonical;
        }
        ancestor = parent;
    }
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
#[path = "importers_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "importers/tests.rs"]
mod directory_tests;
