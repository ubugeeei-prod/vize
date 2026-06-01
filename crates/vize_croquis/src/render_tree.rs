//! Render-tree call graph: which parent template renders which child component.
//!
//! Foundation for issue #694. The graph powers cross-file diagnostics like
//! "required prop X is never passed by any caller" and Musea coverage maps.
//!
//! The actual graph build happens incrementally per-SFC in the cross-file
//! analyzer. This module ships the data model so the build and consumers can
//! land in follow-ups against the same shape.

use vize_carton::CompactString;

use crate::scope::Span;

/// One usage edge: parent template references a child component tag.
#[derive(Debug, Clone)]
pub struct RenderEdge {
    /// Component name used in the parent template (PascalCase or kebab).
    pub child: CompactString,
    /// Source span of the tag in the parent SFC.
    pub span: Span,
    /// Names of the props passed at this usage site (after kebab→camel
    /// normalization). The actual prop values live in the host SFC analysis;
    /// the graph only carries names so cross-file rules can check whether a
    /// required prop made it in.
    pub passed_props: Vec<CompactString>,
    /// Whether the usage carries `v-bind="..."` spread attrs. When true the
    /// consumer should treat any prop as potentially passed.
    pub has_spread: bool,
}

/// Render-tree edges discovered in one parent SFC. The cross-file analyzer
/// merges these across the workspace.
#[derive(Debug, Default, Clone)]
pub struct RenderTreeFragment {
    /// Component name of the parent (deduced from the file path, e.g.
    /// `Button.vue` → `Button`).
    pub parent: CompactString,
    /// Child usages.
    pub edges: Vec<RenderEdge>,
}

impl RenderTreeFragment {
    /// Create an empty fragment for `parent`.
    pub fn new(parent: impl Into<CompactString>) -> Self {
        Self {
            parent: parent.into(),
            edges: Vec::new(),
        }
    }

    /// Iterate all edges that reference `child`.
    pub fn edges_to(&self, child: &str) -> impl Iterator<Item = &RenderEdge> {
        self.edges
            .iter()
            .filter(move |edge| edge.child.as_str() == child)
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderEdge, RenderTreeFragment};
    use crate::scope::Span;
    use vize_carton::CompactString;

    #[test]
    fn fragment_groups_edges_by_child() {
        let mut fragment = RenderTreeFragment::new("Parent");
        fragment.edges.push(RenderEdge {
            child: CompactString::new("MyButton"),
            span: Span::new(0, 10),
            passed_props: vec![CompactString::new("label")],
            has_spread: false,
        });
        fragment.edges.push(RenderEdge {
            child: CompactString::new("MyButton"),
            span: Span::new(20, 30),
            passed_props: vec![],
            has_spread: true,
        });
        fragment.edges.push(RenderEdge {
            child: CompactString::new("MyInput"),
            span: Span::new(40, 50),
            passed_props: vec![],
            has_spread: false,
        });

        assert_eq!(fragment.edges_to("MyButton").count(), 2);
        assert_eq!(fragment.edges_to("MyInput").count(), 1);
        assert_eq!(fragment.edges_to("MyMissing").count(), 0);
    }
}
