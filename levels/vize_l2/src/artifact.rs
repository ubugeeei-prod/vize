//! Native canonical L2 ownership, independent of a dialect legalizer.
//!
//! Construction consumes the existing region and retained `ExprRef`
//! payloads: it neither parses expressions nor materializes a dump. The
//! immutable owner seals page order, so scopes and provenance cannot be
//! silently detached by mutation. Consuming it with `into_parts` ends
//! that seal; a producer must check its changed parts again.
//!
//! This is the native provider boundary, not proof that the transitional
//! Vue lowering, L1 parsed-embed handoff, or product routes use it yet.

use alloc::boxed::Box;
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId, side_table::SideTable};

use crate::op::Region;
use crate::provenance::ProvenanceRecord;
use crate::scope::{ScopeFacts, ScopeTag};
use crate::walk::{NodeEvent, NodeLimit, NodeRef, PageWalk};

mod builder;
mod check;
pub(crate) use builder::ElementAllocation;
pub use builder::{Builder, ComponentBody, ComponentFactory, RegionBuilder};
#[cfg(test)]
mod tests;

/// The native fields a producer transfers into L2 ownership.
///
/// Source coordinates are file-absolute UTF-8 bytes. Decoded or synthesized
/// expression text may live in the arena; it need not equal its authored
/// source slice. Dialect wrappers, target choices and legacy capabilities
/// have no fields here.
#[derive(Debug)]
pub struct ArtifactParts<'a> {
    /// The complete authored source, borrowed for this compile.
    pub source: &'a str,
    /// The normalized, arena-resident owned region.
    pub root: Region<'a>,
    /// Decisions in producer order, including failed decisions with no node.
    pub provenance: Vec<ProvenanceRecord>,
    /// Binding introduction facts keyed by exact page-local node ids.
    pub scopes: SideTable<ScopeFacts>,
}

/// A sealed native L2 artifact. It exposes no mutable tree or side tables.
#[derive(Debug)]
pub struct Artifact<'a> {
    parts: ArtifactParts<'a>,
    node_count: u32,
}

/// A failed constructor retains the actual partial tree and all decisions.
///
/// Boxing the parts only on rejection keeps the result's error small; it
/// does not copy arena nodes or expression payloads.
#[derive(Debug)]
pub struct RejectedArtifact<'a> {
    /// The first deterministic invariant failure.
    pub error: ArtifactError,
    /// The original inputs, available for repair or diagnostic reporting.
    pub parts: Box<ArtifactParts<'a>>,
}

/// Which canonical conditional form was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfShape {
    /// A conditional owns no branches.
    Empty,
    /// Its first branch has no condition.
    LeadingElse,
    /// An unconditional branch appears before the last branch.
    NonTrailingElse,
}

/// Checked ownership errors, reported without a compiler panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactError {
    /// The complete source does not fit in the shared u32 coordinate space.
    SourceLimit,
    /// At least one node did not fit in the shared NodeId space.
    NodeLimit,
    /// A producer's child callback unwound before completing an owner.
    UnfinishedOwner { node: NodeId },
    /// An attached binding has no element or component owner.
    BindingWithoutOwner { span: Span },
    /// An attached binding would be numbered after its owner's child region.
    BindingAfterChild { node: NodeId, span: Span },
    /// A static binding name is empty or does not equal its authored range.
    InvalidBindingName { node: NodeId, span: Span },
    /// A direct whole-handler reference names a different attached node.
    InvalidHandlerReference { node: NodeId, handler: NodeId },
    /// A whole original head names a different introducing node.
    InvalidOriginalForReference { node: NodeId, original: NodeId },
    /// An authored span is inverted, outside source, or cuts UTF-8.
    InvalidSpan { node: Option<NodeId>, span: Span },
    /// A node or payload span escapes its immediate source owner.
    OutsideOwner {
        node: NodeId,
        span: Span,
        owner: Span,
    },
    /// A structured conditional is not canonical.
    InvalidIf { node: NodeId, shape: IfShape },
    /// A provenance record refers to a node absent from this artifact.
    DanglingProvenance { record: usize, node: NodeId },
    /// A scope fact refers to a node absent from this artifact.
    DanglingScope { node: NodeId },
    /// A scope fact is attached to a node with no binding introduction site.
    InvalidScopeSite { node: NodeId },
    /// A binding introduction site lacks its scope identity.
    MissingScope { node: NodeId },
    /// A retained JS root span does not index its own expression source.
    InvalidJsSpan { node: NodeId, span: Span },
    /// A native expression's decode coordinates belong to different source bytes.
    MismatchedJsSource { node: NodeId, span: Span },
    /// Scope tags are not a unique dense set starting at zero.
    InvalidScopeTag { node: NodeId, tag: ScopeTag },
}

impl<'a> Artifact<'a> {
    /// Consume native parts, checking their invariants once at creation.
    ///
    /// The node count is derived in the same walk that checks spans,
    /// expressions, branches and scope sites. This is a provider constructor,
    /// not a new pipeline phase, release verifier, or reparsing adapter.
    pub fn try_new(parts: ArtifactParts<'a>) -> Result<Self, RejectedArtifact<'a>> {
        match check::check(&parts) {
            Ok(node_count) => Ok(Self { parts, node_count }),
            Err(error) => Err(RejectedArtifact {
                error,
                parts: Box::new(parts),
            }),
        }
    }

    /// The complete authored source.
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.parts.source
    }

    /// The borrowed native tree. Mutation requires consuming this owner.
    #[must_use]
    pub fn root(&self) -> &Region<'a> {
        &self.parts.root
    }

    /// Every numbered node, including attached bindings.
    #[must_use]
    pub fn node_count(&self) -> u32 {
        self.node_count
    }

    /// Whether this artifact numbers the provided stage-local id.
    ///
    /// NodeId carries no cross-artifact identity. Consumers must retain the
    /// artifact borrow alongside their tables; equal indices in two owners
    /// are not evidence of shared nodes.
    #[must_use]
    pub fn contains_node(&self, node: NodeId) -> bool {
        node.index() < self.node_count
    }

    /// Sparse immutable scope facts; use sorted_entries for printing.
    #[must_use]
    pub fn scopes(&self) -> &SideTable<ScopeFacts> {
        &self.parts.scopes
    }

    /// Producer-ordered provenance, including unsuccessful decisions.
    #[must_use]
    pub fn provenance(&self) -> &[ProvenanceRecord] {
        &self.parts.provenance
    }

    /// Visit each exact numbered node once in page preorder.
    ///
    /// A sealed artifact cannot exhaust the id space. The fallible shared
    /// walk still reports an invariant failure rather than hiding a prefix.
    pub fn visit_nodes<'s>(
        &'s self,
        visit: &mut impl FnMut(NodeId, NodeRef<'s, 'a>),
    ) -> Result<(), NodeLimit> {
        crate::walk::visit_nodes(&mut PageWalk::new(), &self.parts.root.ops, visit)
    }

    /// Nested traversal for bottom-up consumers, with no second id mint.
    pub fn visit_events<'s>(
        &'s self,
        visit: &mut impl FnMut(NodeEvent<'s, 'a>),
    ) -> Result<(), NodeLimit> {
        crate::walk::visit_events(&mut PageWalk::new(), &self.parts.root.ops, visit)
    }

    /// Consume the seal to change the tree or its facts.
    ///
    /// Previously derived tables are invalid after changing page order;
    /// producers must update them and reconstruct an Artifact.
    #[must_use]
    pub fn into_parts(self) -> ArtifactParts<'a> {
        self.parts
    }
}
