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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum RequestedExports {
    All,
    Names(Vec<String>),
}

impl RequestedExports {
    /// The same request with its names in one order, each once: two requests
    /// ask for the same exports exactly when these are equal. A namespace
    /// request stays apart from every list of names, `"*"` included.
    pub(super) fn canonical(&self) -> Self {
        match self {
            Self::All => Self::All,
            Self::Names(names) => {
                let mut names = names.clone();
                names.sort();
                names.dedup();
                Self::Names(names)
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
    requests: FxHashMap<RequestedExports, usize>,
    /// Whether an edge resolved to a module that could not be read.
    unread: bool,
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
            unread: false,
        }
    }

    /// Walk from a module that an import of the root resolved to.
    pub(super) fn enter(&mut self, module: ModuleId, requested: &RequestedExports) {
        self.resolved.insert(module);
        let request = self.request(requested);
        self.walk(module, requested, request);
    }

    /// Every module an edge resolved to, in the order of their dependency
    /// names, and whether all of them could be read. A module that could not
    /// be read is still a dependency.
    pub(super) fn finish(self) -> (Vec<ModuleId>, bool) {
        let mut resolved: Vec<_> = self
            .resolved
            .iter()
            .filter_map(|id| Some((self.graph.module(*id)?.key.as_str(), *id)))
            .collect();
        resolved.sort_unstable();
        let resolved = resolved.into_iter().map(|(_, id)| id).collect();
        (resolved, !self.unread)
    }

    fn request(&mut self, requested: &RequestedExports) -> usize {
        let next = self.requests.len();
        *self.requests.entry(requested.canonical()).or_insert(next)
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
            self.unread = true;
            return;
        };
        if self.visited.insert(id) {
            count(Step::Module, 1);
            self.visit.module(id, &facts.summary);
        }

        let dependencies = facts.summary.dependencies.as_slice();
        // Every star re-export of a module is asked for the same names.
        let star = dependencies
            .iter()
            .any(|dependency| matches!(dependency, TypeDependency::StarReexport { .. }))
            .then(|| {
                let star_requested = requested.without_named_reexports(dependencies);
                let star_request = self.request(&star_requested);
                (star_requested, star_request)
            });
        let mut followed = Vec::new();
        for (dependency, edge) in dependencies.iter().zip(facts.edges.iter()) {
            count(Step::Edge, 1);
            let (next, next_request) = match (&star, dependency) {
                (Some((star_requested, star_request)), TypeDependency::StarReexport { .. }) => {
                    let Some(next) = dependency.request(star_requested) else {
                        continue;
                    };
                    (next, *star_request)
                }
                _ => {
                    let Some(next) = dependency.request(requested) else {
                        continue;
                    };
                    let next_request = self.request(&next);
                    (next, next_request)
                }
            };
            followed.push((dependency, edge, next, next_request));
        }
        // Workers walking this module together read different targets instead
        // of waiting one behind the other on each. The order of the walk
        // below, and so of the merge, does not change.
        if followed.len() > 1 && graph.is_shared() {
            for (dependency, edge, ..) in &followed {
                graph.read_ahead(module, dependency, edge, self.project);
            }
        }
        for (dependency, edge, next, next_request) in followed {
            let Some(target) = graph.target(module, dependency, edge, self.project) else {
                continue;
            };
            self.resolved.insert(target);
            self.walk(target, &next, next_request);
        }
    }
}
