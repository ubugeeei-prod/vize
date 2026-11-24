//! What a walk from one module gives every component that reaches it.

use super::FileTypeSummary;
use super::modules::{ModuleGraph, ModuleId};
use super::resolution::project::Project;
use super::walk::{RequestedExports, Visit, Walk};

/// The outcome of walking from one module for one set of requested exports.
/// It names no importer, so the components that reach the module the same way
/// share one closure, and it holds no text: the modules stay in the graph.
#[derive(Debug, Default)]
pub(super) struct TypeClosure {
    /// The modules the walk visited, in visit order. A component merges
    /// their summaries in this order, as its own walk would have.
    pub(super) modules: Vec<ModuleId>,
    /// Every module the walk resolved, in the order of their dependency
    /// names.
    pub(super) files: Vec<ModuleId>,
    /// Whether every resolved module could be read. A walk that met an
    /// unreadable one says nothing about the next attempt.
    pub(super) complete: bool,
}

impl TypeClosure {
    pub(super) fn walk(
        graph: &ModuleGraph,
        module: ModuleId,
        requested: &RequestedExports,
        project: Option<&Project>,
    ) -> Self {
        let mut modules = Vec::new();
        let mut walk = Walk::new(graph, project, &mut modules);
        walk.enter(module, requested);
        let (files, complete) = walk.finish();
        Self {
            modules,
            files,
            complete,
        }
    }
}

impl Visit for Vec<ModuleId> {
    fn module(&mut self, id: ModuleId, _summary: &FileTypeSummary) {
        self.push(id);
    }
}
