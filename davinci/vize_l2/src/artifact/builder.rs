//! Checked construction seals the native tree during its producer's own walk.

use alloc::vec::Vec;
use vize_l0::{Allocator, Box, Span, id::NodeId, side_table::SideTable};

use super::{Artifact, ArtifactError, ArtifactParts, RejectedArtifact, check};
use crate::expr::ExprRef;
use crate::op::{
    Attribute, BindingOp, CommentOp, ComponentOp, ElementOp, InterpolationOp, Namespace, Op,
    Region, TextOp,
};
use crate::provenance::ProvenanceRecord;
use crate::walk::PageWalk;

struct Frame<'a> {
    id: NodeId,
    owner: Owner<'a>,
    span: Span,
    bindings: vize_l0::Vec<'a, BindingOp<'a>>,
    children_started: bool,
    ops: vize_l0::Vec<'a, Op<'a>>,
}

enum Owner<'a> {
    Element {
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
    },
    Component {
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
    },
}

mod binding;
mod factory;
mod region;
pub use factory::{ComponentBody, ComponentFactory};
pub use region::RegionBuilder;

/// A restricted canonical producer, with no arbitrary region or table insertion.
///
/// The first native slice supports ordinary elements/components and text,
/// comments, retained-expression interpolations and static named bindings.
/// Attached bindings must precede children. Every constructor checks
/// ownership and mints its page id before descending into children. Scope and
/// control-flow factories require their own checked contracts before extension.
/// `finish` directly seals this accounting; it never walks or reparses the tree.
pub struct Builder<'a> {
    allocator: &'a Allocator,
    parts: ArtifactParts<'a>,
    frames: Vec<Frame<'a>>,
    walk: PageWalk,
}

impl<'a> Builder<'a> {
    /// Borrow the root factory scope without exposing owned construction state.
    pub fn region(&mut self) -> RegionBuilder<'_, 'a> {
        RegionBuilder { builder: self }
    }

    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, ArtifactError> {
        u32::try_from(source.len()).map_err(|_| ArtifactError::SourceLimit)?;
        Ok(Self {
            allocator,
            parts: ArtifactParts {
                source,
                root: Region {
                    ops: vize_l0::Vec::new_in(&allocator),
                },
                provenance: Vec::new(),
                scopes: SideTable::new(),
            },
            frames: Vec::new(),
            walk: PageWalk::new(),
        })
    }

    /// Build a leaf only after its span is checked against the current owner.
    pub fn text(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        self.mint();
        self.push(Op::Text(Box::new_in(
            TextOp { content, span },
            &self.allocator,
        )));
        Ok(id)
    }

    pub fn comment(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        self.mint();
        self.push(Op::Comment(Box::new_in(
            CommentOp { content, span },
            &self.allocator,
        )));
        Ok(id)
    }

    pub fn interpolation(
        &mut self,
        expression: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        check::expression(self.parts.source, id, expression, span)?;
        self.mint();
        self.push(Op::Interpolation(Box::new_in(
            InterpolationOp { expression, span },
            &self.allocator,
        )));
        Ok(id)
    }

    /// Create one native owner, then build children in its restricted context.
    /// The closure sees the owner id so provenance can preserve decision order.
    pub fn element(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare_owner(&attributes, span)?;
        self.children(
            Owner::Element {
                tag,
                namespace,
                attributes,
            },
            span,
            id,
            children,
        )?;
        Ok(id)
    }

    pub fn component(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare_owner(&attributes, span)?;
        self.children(Owner::Component { name, attributes }, span, id, children)?;
        Ok(id)
    }

    /// Failed decisions have no node and still retain source and authored order.
    pub fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        check::span(self.parts.source, record.node, record.span)?;
        if let Some(node) = record.node
            && node.index() >= self.walk.minted()
        {
            return Err(ArtifactError::DanglingProvenance {
                record: self.parts.provenance.len(),
                node,
            });
        }
        self.parts.provenance.push(record);
        Ok(())
    }

    /// Seal only factory-created nodes. There is no validation/counting walk.
    pub fn finish(mut self) -> Result<Artifact<'a>, RejectedArtifact<'a>> {
        if let Some(node) = self.frames.last().map(|frame| frame.id) {
            // After a caught callback unwind, retain every partial child and
            // pending owner. Closing only pending frames is not a tree walk.
            while let Some(frame) = self.frames.pop() {
                self.close(frame);
            }
            return Err(RejectedArtifact {
                error: ArtifactError::UnfinishedOwner { node },
                parts: alloc::boxed::Box::new(self.parts),
            });
        }
        Ok(Artifact {
            parts: self.parts,
            node_count: self.walk.minted(),
        })
    }

    fn prepare(&self, span: Span) -> Result<NodeId, ArtifactError> {
        let id = NodeId::from_index(self.walk.minted()).ok_or(ArtifactError::NodeLimit)?;
        check::owned_span(
            self.parts.source,
            id,
            span,
            self.frames.last().map(|frame| frame.span),
        )?;
        Ok(id)
    }

    fn prepare_owner(
        &self,
        attributes: &[Attribute<'a>],
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        for attribute in attributes {
            check::owned_span(self.parts.source, id, attribute.span, Some(span))?;
        }
        Ok(id)
    }

    fn mint(&mut self) {
        // prepare checked this exact next index; no mutation occurs between them.
        if let Some(frame) = self.frames.last_mut() {
            frame.children_started = true;
        }
        let _ = self.walk.mint();
    }

    fn children(
        &mut self,
        owner: Owner<'a>,
        span: Span,
        id: NodeId,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        self.mint();
        self.frames.push(Frame {
            id,
            owner,
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        children(&mut RegionBuilder { builder: self }, id);
        // A callback can catch a nested owner's unwind. Do not close that
        // pending nested frame or report this outer owner as complete.
        let pending = self.frames.last().map(|frame| frame.id);
        if pending != Some(id) {
            return Err(ArtifactError::UnfinishedOwner {
                node: pending.unwrap_or(id),
            });
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node: id })?;
        self.close(frame);
        Ok(())
    }

    fn close(&mut self, frame: Frame<'a>) {
        let children = Region { ops: frame.ops };
        let bindings = frame.bindings;
        let span = frame.span;
        let op = match frame.owner {
            Owner::Element {
                tag,
                namespace,
                attributes,
            } => Op::Element(Box::new_in(
                ElementOp {
                    tag,
                    namespace,
                    attributes,
                    bindings,
                    children,
                    span,
                },
                &self.allocator,
            )),
            Owner::Component { name, attributes } => Op::Component(Box::new_in(
                ComponentOp {
                    name,
                    attributes,
                    bindings,
                    children,
                    span,
                },
                &self.allocator,
            )),
        };
        self.push(op);
    }

    fn push(&mut self, op: Op<'a>) {
        match self.frames.last_mut() {
            Some(frame) => frame.ops.push(op),
            None => self.parts.root.ops.push(op),
        }
    }
}

#[cfg(test)]
mod tests;
