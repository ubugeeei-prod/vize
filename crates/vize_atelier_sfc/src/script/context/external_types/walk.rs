//! Selective traversal of imported type module edges.

use std::borrow::Cow;

use vize_carton::{FxHashMap, FxHashSet, String};

use super::FileTypeSummary;
use super::modules::{ModuleGraph, ModuleId};
use super::resolution::project::Project;
use super::work::{Step, count};

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

impl TypeDependency {
    pub(super) fn specifier(&self) -> &str {
        match self {
            Self::Import { specifier, .. }
            | Self::NamedReexport { specifier, .. }
            | Self::StarReexport { specifier } => specifier,
        }
    }

    /// What this edge asks of its target when its own module is asked for
    /// `requested`, or `None` when the edge is not followed. A star re-export
    /// is asked for the names its module's named re-exports left over.
    pub(super) fn request<'a>(
        &self,
        requested: &'a RequestedExports,
    ) -> Option<Cow<'a, RequestedExports>> {
        if requested.is_empty() {
            return None;
        }
        match self {
            Self::Import { names, .. } => Some(Cow::Owned(
                names.as_ref().map_or(RequestedExports::All, |names| {
                    RequestedExports::Names(names.clone())
                }),
            )),
            Self::NamedReexport { bindings, .. } => {
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
                (!names.is_empty()).then(|| Cow::Owned(RequestedExports::Names(names)))
            }
            Self::StarReexport { .. } => Some(Cow::Borrowed(requested)),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum RequestedExports {
    All,
    Names(Vec<String>),
}

impl RequestedExports {
    pub(super) fn cache_key(&self) -> String {
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

    pub(super) fn without_named_reexports(&self, dependencies: &[TypeDependency]) -> Self {
        let Self::Names(names) = self else {
            return Self::All;
        };
        let mut remaining = names.clone();
        remaining.retain(|name| {
            !dependencies.iter().any(|dependency| {
                let TypeDependency::NamedReexport { bindings, .. } = dependency else {
                    return false;
                };
                bindings.iter().any(|(exported, _)| exported == name)
            })
        });
        Self::Names(remaining)
    }

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Self::Names(names) if names.is_empty())
    }
}

/// What a walk does with each module it visits. Modules arrive once, in the
/// order a component merges them: the first declaration of a name wins.
pub(super) trait Visit {
    fn module(&mut self, id: ModuleId, summary: &FileTypeSummary);
}

/// One depth-first walk over the modules of a graph. A walk follows the same
/// edges in the same order whoever the importer is.
pub(super) struct Walk<'a, V> {
    graph: &'a ModuleGraph,
    project: Option<&'a Project>,
    visit: &'a mut V,
    /// Modules a followed edge resolved to.
    resolved: FxHashSet<ModuleId>,
    /// Modules already handed to `visit`.
    visited: FxHashSet<ModuleId>,
    /// Modules already walked, with the number of the request they were
    /// walked for.
    traversed: FxHashSet<(ModuleId, usize)>,
    /// The requested export sets this walk has met, numbered.
    requests: FxHashMap<String, usize>,
}

impl<'a, V: Visit> Walk<'a, V> {
    pub(super) fn new(
        graph: &'a ModuleGraph,
        project: Option<&'a Project>,
        visit: &'a mut V,
    ) -> Self {
        Self {
            graph,
            project,
            visit,
            resolved: FxHashSet::default(),
            visited: FxHashSet::default(),
            traversed: FxHashSet::default(),
            requests: FxHashMap::default(),
        }
    }

    /// Walk from a module that an import of the root resolved to.
    pub(super) fn enter(&mut self, module: ModuleId, requested: &RequestedExports) {
        self.resolved.insert(module);
        let request = self.request(requested);
        self.walk(module, requested, request);
    }

    /// Every module an edge resolved to, in the order of their dependency
    /// names. A module that could not be read is still a dependency.
    pub(super) fn finish(self) -> Vec<ModuleId> {
        let mut resolved: Vec<_> = self
            .resolved
            .iter()
            .filter_map(|id| Some((self.graph.module(*id)?.key.as_str(), *id)))
            .collect();
        resolved.sort_unstable();
        resolved.into_iter().map(|(_, id)| id).collect()
    }

    fn request(&mut self, requested: &RequestedExports) -> usize {
        let next = self.requests.len();
        *self.requests.entry(requested.cache_key()).or_insert(next)
    }

    fn walk(&mut self, id: ModuleId, requested: &RequestedExports, request: usize) {
        if !self.traversed.insert((id, request)) {
            return;
        }
        let graph = self.graph;
        let Some(module) = graph.module(id) else {
            return;
        };
        let Some(facts) = module.facts() else {
            return;
        };
        if self.visited.insert(id) {
            count(Step::Module, 1);
            self.visit.module(id, &facts.summary);
        }

        let dependencies = facts.summary.dependencies.as_slice();
        // Every star re-export of a module is asked for the same names.
        let mut star: Option<(RequestedExports, usize)> = None;
        for (dependency, target) in dependencies.iter().zip(facts.targets.iter()) {
            count(Step::Edge, 1);
            let (next, next_request) = if matches!(dependency, TypeDependency::StarReexport { .. })
            {
                let (star_requested, star_request) = star.get_or_insert_with(|| {
                    let star_requested = requested.without_named_reexports(dependencies);
                    let star_request = self.request(&star_requested);
                    (star_requested, star_request)
                });
                let Some(next) = dependency.request(star_requested) else {
                    continue;
                };
                (next, *star_request)
            } else {
                let Some(next) = dependency.request(requested) else {
                    continue;
                };
                let next_request = self.request(&next);
                (next, next_request)
            };
            let Some(target) = graph.target(module, dependency, target, self.project) else {
                continue;
            };
            self.resolved.insert(target);
            self.walk(target, &next, next_request);
        }
    }
}
