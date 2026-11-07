//! Selective traversal of imported type module edges.

use std::path::Path;

use vize_carton::{FxHashSet, String};

use super::super::batch_epoch::current_batch_epoch;
use super::resolution::{path_key, project::Project, resolve_import_path_with_project};
use super::{FileTypeSource, ScriptCompileContext, SummaryLookup};

#[derive(Clone, Debug)]
pub(super) enum TypeDependency {
    Import {
        specifier: String,
        /// `None` means a namespace import and therefore the whole module.
        names: Option<Vec<String>>,
    },
    NamedReexport {
        specifier: String,
        /// Pairs of the exported name and the name in the target module.
        bindings: Vec<(String, String)>,
    },
    StarReexport {
        specifier: String,
    },
}

#[derive(Clone, Debug)]
pub(super) enum RequestedExports {
    All,
    Names(Vec<String>),
}

impl RequestedExports {
    fn cache_key(&self) -> String {
        match self {
            Self::All => String::from("*"),
            Self::Names(names) => {
                let mut names = names.clone();
                names.sort();
                names.dedup();
                String::from(names.join("\0"))
            }
        }
    }
}

pub(super) struct TypeWalk<'a> {
    pub visited: &'a mut FxHashSet<String>,
    pub traversed: &'a mut FxHashSet<String>,
    pub project: Option<&'a Project>,
    pub imports: &'a mut Vec<(String, String)>,
}

impl ScriptCompileContext {
    pub(super) fn collect_types_from_dependency(
        &mut self,
        dependency: &TypeDependency,
        requested: &RequestedExports,
        current_file: &Path,
        walk: &mut TypeWalk<'_>,
    ) {
        let (specifier, requested) = match dependency {
            TypeDependency::Import { specifier, names } => (
                specifier,
                names.as_ref().map_or(RequestedExports::All, |names| {
                    RequestedExports::Names(names.clone())
                }),
            ),
            TypeDependency::NamedReexport {
                specifier,
                bindings,
            } => {
                let names = match requested {
                    RequestedExports::All => {
                        bindings.iter().map(|(_, local)| local.clone()).collect()
                    }
                    RequestedExports::Names(requested) => bindings
                        .iter()
                        .filter(|(exported, _)| requested.contains(exported))
                        .map(|(_, local)| local.clone())
                        .collect::<Vec<_>>(),
                };
                if names.is_empty() {
                    return;
                }
                (specifier, RequestedExports::Names(names))
            }
            TypeDependency::StarReexport { specifier } => (specifier, requested.clone()),
        };
        self.collect_types_from_specifier(specifier, &requested, current_file, walk);
    }

    fn collect_types_from_specifier(
        &mut self,
        specifier: &str,
        requested: &RequestedExports,
        current_file: &Path,
        walk: &mut TypeWalk<'_>,
    ) {
        let Some(resolved_path) =
            resolve_import_path_with_project(current_file, specifier, walk.project)
        else {
            return;
        };

        let key = path_key(&resolved_path);
        walk.visited.insert(key.clone());
        let traversal_key = vize_carton::cstr!("{key}\0{}", requested.cache_key());
        if !walk.traversed.insert(traversal_key) {
            return;
        }

        let epoch = current_batch_epoch();
        let dependencies = match self.cached_file_summary(&resolved_path, epoch, walk.imports) {
            SummaryLookup::Cached(dependencies) => dependencies,
            SummaryLookup::Reload(source) => {
                let Some(dependencies) =
                    self.merge_file_source(&resolved_path, source, epoch, walk.imports)
                else {
                    return;
                };
                dependencies
            }
            SummaryLookup::Miss => {
                let source = FileTypeSource::read(&resolved_path);
                let Some(dependencies) =
                    self.merge_file_source(&resolved_path, source, epoch, walk.imports)
                else {
                    return;
                };
                dependencies
            }
        };

        for dependency in &dependencies {
            self.collect_types_from_dependency(dependency, requested, &resolved_path, walk);
        }
    }
}
