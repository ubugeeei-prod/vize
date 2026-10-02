//! Private same-recorder bridge into the sole retained native construction walk.

use super::Visibility;
use crate::native::{ConstructionBody, ConstructionFactory};
use vize_l0::{Span, id::NodeId};
use vize_l2::artifact::{ArtifactError, ComponentFactory};
use vize_l2::expr::JsExpr;
use vize_l2::file::{TemplateBody, TemplateChildRegion, TemplateWalkRegion};
use vize_l2::op::{Attribute, Namespace};
use vize_l2::provenance::ProvenanceRecord;

pub(super) struct VueNative<'r, 'g, 'f, 'a> {
    region: &'r mut TemplateWalkRegion<'g, 'f, 'a, Visibility>,
}
impl<'r, 'g, 'f, 'a> VueNative<'r, 'g, 'f, 'a> {
    pub(super) fn new(region: &'r mut TemplateWalkRegion<'g, 'f, 'a, Visibility>) -> Self {
        Self { region }
    }
}
impl<'a> ConstructionFactory<'a> for VueNative<'_, '_, '_, 'a> {
    fn source(&self) -> &'a str {
        self.region.source()
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.region.interpolation(expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.region.bind(name, name_span, expression, span)
    }
    fn element<B: ConstructionBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region
            .element_with(tag, namespace, attributes, span, NativeBody(body))
    }
    fn component<B: ConstructionBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region
            .component_with(name, attributes, span, NativeBody(body))
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.region.record(record)
    }
}

struct NativeChild<'c, 'r, 'b, 'a> {
    region: &'c mut TemplateChildRegion<'r, 'b, 'a, Visibility>,
}
impl<'a> ConstructionFactory<'a> for NativeChild<'_, '_, '_, 'a> {
    fn source(&self) -> &'a str {
        self.region.source()
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.region.interpolation(expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.region.bind(name, name_span, expression, span)
    }
    fn element<B: ConstructionBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region
            .element_with(tag, namespace, attributes, span, NativeBody(body))
    }
    fn component<B: ConstructionBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region
            .component_with(name, attributes, span, NativeBody(body))
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.region.record(record)
    }
}

struct NativeBody<B>(B);
impl<'a, B: ConstructionBody<'a>> TemplateBody<'a, Visibility> for NativeBody<B> {
    fn run<'r, 'b>(self, region: &mut TemplateChildRegion<'r, 'b, 'a, Visibility>, node: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        self.0.run(&mut NativeChild { region }, node);
    }
}
