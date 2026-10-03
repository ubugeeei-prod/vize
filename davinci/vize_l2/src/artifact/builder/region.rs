//! Borrowed factory scope; callers cannot replace or finish its owning builder.

use super::{ArtifactError, Builder};
use crate::expr::ExprRef;
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use vize_l0::{Span, id::NodeId};

/// A child region with access only to checked node factories and provenance.
pub struct RegionBuilder<'s, 'a> {
    pub(super) builder: &'s mut Builder<'a>,
}

impl<'a> RegionBuilder<'_, 'a> {
    pub(crate) fn begin_original_for(
        &mut self,
        head: &crate::resolution::ForResolution<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.builder.begin_original_for(head, span)
    }
    pub(crate) fn finish_original_for(
        &mut self,
        node: NodeId,
    ) -> Result<core::ptr::NonNull<crate::op::OriginalForOp<'a>>, ArtifactError> {
        self.builder.finish_original_for(node)
    }
    pub(crate) fn native_on(
        &mut self,
        handler: &crate::resolution::HandlerResolution<'a>,
        span: Span,
    ) -> Result<(NodeId, core::ptr::NonNull<crate::op::OnOp<'a>>), ArtifactError> {
        self.builder.native_on(handler, span)
    }
    /// Attach an explicit static named binding before this owner's children.
    pub fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.builder.bind(name, name_span, value, span)
    }

    pub fn text(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.builder.text(content, span)
    }
    pub fn comment(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.builder.comment(content, span)
    }
    pub fn interpolation(
        &mut self,
        expression: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.builder.interpolation(expression, span)
    }
    pub fn element(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        self.builder
            .element(tag, namespace, attributes, span, children)
    }
    pub fn component(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        self.builder.component(name, attributes, span, children)
    }
    pub fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.builder.record(record)
    }
}
