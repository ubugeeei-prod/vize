//! Reverse dependency storage for open Vue importers.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_carton::{FxHashMap, FxHashSet};

use super::{collect_dependencies, comparable_path};

#[derive(Default)]
pub(in crate::server) struct OpenVueImportIndex {
    inner: RwLock<ImportIndexData>,
}

#[derive(Default)]
struct ImportIndexData {
    by_dependency: BTreeMap<PathBuf, FxHashSet<Url>>,
    by_importer: FxHashMap<Url, Vec<PathBuf>>,
}

impl OpenVueImportIndex {
    pub(in crate::server) fn update(&self, importer: &Url, source: &str) {
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

    pub(in crate::server) fn remove(&self, importer: &Url) {
        remove_importer(&mut self.inner.write(), importer);
    }

    pub(in crate::server) fn clear(&self) {
        let mut index = self.inner.write();
        index.by_dependency.clear();
        index.by_importer.clear();
    }

    pub(super) fn importers(&self, dependency: &Path) -> Vec<Url> {
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
    pub(super) fn dependency_paths(&self, dependency: &Path) -> Vec<PathBuf> {
        let dependency = comparable_path(dependency);
        let index = self.inner.read();
        let mut paths = index
            .by_dependency
            .range(dependency.clone()..)
            .take_while(|(path, _)| path.starts_with(&dependency))
            .map(|(path, _)| path)
            .cloned()
            .collect::<Vec<_>>();
        paths.sort();
        paths
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
