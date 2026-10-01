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
