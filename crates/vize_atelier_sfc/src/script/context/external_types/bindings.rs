//! The declaration each named type import of a component binds.
//!
//! The walks merge every module they reach into flat maps, where the first
//! declaration of a name wins. A component's own import names one export of
//! one module, though: `import type { Public as Alias } from './types'` binds
//! `Alias` to what `./types` exports as `Public`, and `import type { Item }
//! from './detail'` binds `Item` to `./detail`'s, whatever another imported
//! module declares under that name. These bindings merge before the walks, so
//! the first declaration of each imported name is the one its import names.

#![expect(
    clippy::disallowed_types,
    reason = "the components of a batch share one walk's declarations"
)]

use std::sync::Arc;

use vize_carton::{FxHashSet, String};

use super::declarations::Declarations;
use super::modules::{ModuleGraph, ModuleId};
use super::resolution::project::Project;
use super::walk::TypeDependency;

/// A named import of a script: the name it binds, the export it names and the
/// index of its import among the script's dependencies.
#[derive(Clone, Debug)]
pub(super) struct ImportBinding {
    pub(super) local: String,
    pub(super) imported: String,
    pub(super) dependency: usize,
}

/// Where a script summary hands its named imports.
pub(super) trait BindingSink {
    fn len(&self) -> usize;
    fn bind(&mut self, local: &str, imported: &String, dependency: usize);
    fn truncate(&mut self, len: usize);
}

/// The imports of a module the walks read bind nothing a component looks up.
impl BindingSink for () {
    #[inline(always)]
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn bind(&mut self, _local: &str, _imported: &String, _dependency: usize) {}

    #[inline(always)]
    fn truncate(&mut self, _len: usize) {}
}

/// A component's own script keeps every named import.
impl BindingSink for Vec<ImportBinding> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn bind(&mut self, local: &str, imported: &String, dependency: usize) {
        self.push(ImportBinding {
            local: local.into(),
            imported: imported.clone(),
            dependency,
        });
    }

    fn truncate(&mut self, len: usize) {
        Vec::truncate(self, len);
    }
}

/// Where a module's export is declared: the declaring module, and which of its
/// interfaces or type aliases it is.
#[derive(Clone, Copy, Debug)]
pub(super) struct Export {
    pub(super) module: ModuleId,
    pub(super) interface: bool,
    pub(super) index: usize,
}

impl Export {
    /// The name and body of the declaration in `graph`, which resolved it.
    pub(super) fn declaration<'g>(&self, graph: &'g ModuleGraph) -> Option<&'g (String, String)> {
        let summary = graph.summary(self.module)?;
        let declarations = if self.interface {
            &summary.interfaces
        } else {
            &summary.type_aliases
        };
        declarations.get(self.index)
    }
}

/// A local name bound to the export of one walk's [`Declarations`].
#[derive(Debug)]
pub(crate) struct Bound {
    pub(super) local: String,
    pub(super) walk: Arc<Declarations>,
    pub(super) export: usize,
}

/// The declaration `module` exports as `name`: its own first, then the one a
/// named re-export forwards, then the first one a star re-export reaches.
/// `None` when the summaries do not show it, and the walks decide the name.
pub(super) fn resolve(
    graph: &ModuleGraph,
    module: ModuleId,
    name: &str,
    project: Option<&Project>,
) -> Option<Export> {
    let mut seen = FxHashSet::default();
    resolve_in(graph, module, name, project, &mut seen)
}

fn resolve_in(
    graph: &ModuleGraph,
    id: ModuleId,
    name: &str,
    project: Option<&Project>,
    seen: &mut FxHashSet<(ModuleId, String)>,
) -> Option<Export> {
    let module = graph.module(id)?;
    let facts = module.facts()?;
    let summary = &facts.summary;
    // A module's interfaces merge before its type aliases.
    let own = |interface: bool, declarations: &[(String, String)]| {
        let index = declarations
            .iter()
            .position(|(declared, _)| declared == name)?;
        Some(Export {
            module: id,
            interface,
            index,
        })
    };
    if let Some(export) =
        own(true, &summary.interfaces).or_else(|| own(false, &summary.type_aliases))
    {
        return Some(export);
    }
    // Only a name declared further on needs the set: re-exports may cycle.
    if !seen.insert((id, name.into())) {
        return None;
    }
    let dependencies = summary.dependencies.iter().zip(facts.edges.iter());
    for (dependency, edge) in dependencies.clone() {
        let TypeDependency::NamedReexport { bindings, .. } = dependency else {
            continue;
        };
        let Some((_, local)) = bindings.iter().find(|(exported, _)| exported == name) else {
            continue;
        };
        // An explicit re-export of the name is its export, resolved or not.
        let target = graph.target(module, dependency, edge, project)?;
        return resolve_in(graph, target, local, project, seen);
    }
    // `export *` forwards every export but the default one.
    if name == "default" {
        return None;
    }
    for (dependency, edge) in dependencies {
        if !matches!(dependency, TypeDependency::StarReexport { .. }) {
            continue;
        }
        if let Some(export) = graph
            .target(module, dependency, edge, project)
            .and_then(|target| resolve_in(graph, target, name, project, seen))
        {
            return Some(export);
        }
    }
    None
}
