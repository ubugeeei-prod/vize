//! Rendered template complexity (Davinci P4-9b): a component's own score
//! plus the own score of every **distinct** component reachable in its
//! render tree through template component-usage edges.
//!
//! Reachability is a set, so a child shared by two branches, or by two
//! parents, counts once per root, and a strongly connected component (a
//! recursive component, or mutual recursion) is counted once: its members
//! are all reachable from each other, and each is summed exactly once. A
//! child never changes its parent's **own** score; only this cross-file
//! number grows with the tree.

use vize_carton::{CompactString, FxHashMap, FxHashSet};

use super::template::{TemplateComplexity, TemplateScores};
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};

/// One component's own and rendered template complexity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentComplexity {
    pub file_id: FileId,
    pub file_name: CompactString,
    pub component_name: Option<CompactString>,
    /// Own facts and their breakdown.
    pub template: TemplateComplexity,
    /// Own plus every distinct reachable child's own score.
    pub rendered: TemplateScores,
    /// Distinct components in the render tree, this one excluded.
    pub rendered_components: u32,
    /// Those components, ascending: the files a rendered score depends on.
    pub render_tree: Vec<FileId>,
    /// The component reaches itself (direct or mutual recursion).
    pub recursive: bool,
}

/// Every component with template facts, ordered by rendered cognitive
/// complexity (then rendered cyclomatic, then file name).
pub(crate) fn summarize_template_complexity(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    facts: &FxHashMap<FileId, TemplateComplexity>,
) -> Vec<ComponentComplexity> {
    let mut components: Vec<ComponentComplexity> = registry
        .vue_components()
        .filter_map(|entry| {
            let template = facts.get(&entry.id)?;
            let (reachable, recursive) = render_tree(graph, entry.id);
            let rendered = reachable
                .iter()
                .filter_map(|id| facts.get(id))
                .fold(template.own, |sum, child| sum.add(child.own));
            let mut render_tree: Vec<FileId> = reachable.into_iter().collect();
            render_tree.sort_unstable_by_key(|id| id.as_u32());
            Some(ComponentComplexity {
                file_id: entry.id,
                file_name: entry.filename.clone(),
                component_name: entry.component_name.clone(),
                template: template.clone(),
                rendered,
                rendered_components: u32::try_from(render_tree.len()).unwrap_or(u32::MAX),
                render_tree,
                recursive,
            })
        })
        .collect();
    components.sort_by(|left, right| {
        right
            .rendered
            .cognitive
            .cmp(&left.rendered.cognitive)
            .then(right.rendered.cyclomatic.cmp(&left.rendered.cyclomatic))
            .then_with(|| left.file_name.cmp(&right.file_name))
    });
    components
}

/// The distinct components reachable from `root` (itself excluded) and
/// whether `root` reaches itself.
fn render_tree(graph: &DependencyGraph, root: FileId) -> (FxHashSet<FileId>, bool) {
    let mut reachable = FxHashSet::default();
    let mut recursive = false;
    let mut work = vec![root];
    while let Some(id) = work.pop() {
        for (child, edge) in graph.dependencies(id) {
            if edge != DependencyEdge::ComponentUsage {
                continue;
            }
            if child == root {
                recursive = true;
                continue;
            }
            if reachable.insert(child) {
                work.push(child);
            }
        }
    }
    (reachable, recursive)
}
