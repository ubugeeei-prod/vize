//! What one registered editor source contributes to the rest of its graph.
//!
//! The reachable walk, package-route discovery, resolution stamps and the
//! importers' rewrites read a source only through these facts. An edit that
//! keeps them equal leaves every other file of the revision unchanged.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::String;

use super::{VirtualFile, VirtualProject};
use crate::PackageResolutionMode;
use crate::batch::import_rewriter::ModuleSpecifierFactsCache;

type Occurrences = Vec<(String, PackageResolutionMode)>;

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct EditorSourceFacts {
    passthrough_paths: Vec<PathBuf>,
    artifacts: Vec<ArtifactFacts>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ArtifactFacts {
    virtual_path: PathBuf,
    registered: Occurrences,
    module: Occurrences,
    resolution: Occurrences,
    references: Vec<String>,
}

impl VirtualProject {
    /// `None` when `source` is not registered.
    pub(crate) fn editor_source_facts(
        &self,
        source: &Path,
        facts: &mut ModuleSpecifierFactsCache,
    ) -> Option<EditorSourceFacts> {
        let owner = self.source_artifacts.get(source)?;
        let mut passthrough_paths = owner.passthrough_paths.clone();
        passthrough_paths.sort();
        let mut virtual_paths = owner.virtual_paths.clone();
        virtual_paths.sort();
        let artifacts = virtual_paths
            .into_iter()
            .filter_map(|path| self.virtual_files.get(&path))
            .map(|file| {
                let walked = walked_source_type(&file.virtual_path);
                let module = self.walked_source(file);
                let resolution = self
                    .pre_rewrite_code
                    .get(&file.virtual_path)
                    .or_else(|| self.original_contents.get(&file.virtual_path));
                ArtifactFacts {
                    virtual_path: file.virtual_path.clone(),
                    registered: sorted(facts.collect_occurrences(&file.content, walked)),
                    module: sorted(facts.collect_occurrences(module, walked)),
                    resolution: resolution
                        .zip(super::build::source_type_for_path(&file.virtual_path))
                        .map(|(code, kind)| sorted(facts.collect_occurrences(code, kind)))
                        .unwrap_or_default(),
                    references: self.walked_references(file, module),
                }
            })
            .collect();
        Some(EditorSourceFacts {
            passthrough_paths,
            artifacts,
        })
    }

    /// Package-route targets are registered again from disk by every build.
    pub(crate) fn is_package_route_source(&self, source: &Path) -> bool {
        self.package_route_source_owners
            .get(source)
            .is_some_and(|owners| *owners > 0)
    }

    /// The text the reachable walk collects a file's specifiers from.
    pub(super) fn walked_source<'a>(&'a self, file: &'a VirtualFile) -> &'a String {
        self.module_source(file).unwrap_or(&file.content)
    }

    /// Triple-slash path edges: authored SFC text, not its projection, whose
    /// synthetic references would pull in the whole project.
    pub(super) fn walked_references(&self, file: &VirtualFile, walked: &str) -> Vec<String> {
        if file.source_map.sfc_map.is_some() {
            self.original_contents
                .get(&file.virtual_path)
                .map_or_else(Vec::new, |source| {
                    super::dependency_scan::path_references(source)
                })
        } else {
            super::dependency_scan::path_references(walked)
        }
    }
}

pub(super) fn walked_source_type(virtual_path: &Path) -> SourceType {
    if virtual_path
        .extension()
        .is_some_and(|extension| extension == "tsx")
    {
        SourceType::tsx()
    } else {
        SourceType::ts()
    }
}

fn sorted(mut occurrences: Occurrences) -> Occurrences {
    occurrences.sort();
    occurrences.dedup();
    occurrences
}
