//! Opaque lower diagnostic factory; native admission belongs to its upper owner.
use super::{RootRegion, TemplatePolicy};
use crate::artifact::{ArtifactError, ComponentBody, ComponentFactory};
use crate::expr::JsExpr;
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use vize_l0::{Span, id::NodeId};

pub struct TemplateRegion<'f, 'a, P: TemplatePolicy> {
    inner: RootRegion<'f, 'a, P>,
}
impl<'f, 'a, P: TemplatePolicy> TemplateRegion<'f, 'a, P> {
    pub(crate) fn new(inner: RootRegion<'f, 'a, P>) -> Self {
        Self { inner }
    }
}
impl<'a, P: TemplatePolicy> ComponentFactory<'a> for TemplateRegion<'_, 'a, P> {
    fn source(&self) -> &'a str {
        self.inner.source()
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.interpolation(expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.bind(name, name_span, expression, span)
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.element(tag, namespace, attributes, span, body)
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.component(name, attributes, span, body)
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.inner.record(record)
    }
}
