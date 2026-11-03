//! A diagnostic-only test recorder; it grants no file-resolution completeness.

use vize_l0::{Span, id::NodeId};
use vize_l2::artifact::{ArtifactError, ComponentBody, ComponentFactory, RegionBuilder};
use vize_l2::expr::JsExpr;
use vize_l2::op::{Attribute, Namespace};
use vize_l2::provenance::ProvenanceRecord;

pub(super) struct Sites<'a> {
    pub expressions: alloc::vec::Vec<(NodeId, &'a JsExpr<'a>)>,
    pub owners: alloc::vec::Vec<NodeId>,
}

pub(super) struct Observed<'s, 'region, 'a> {
    pub region: &'s mut RegionBuilder<'region, 'a>,
    pub sites: &'s mut Sites<'a>,
}

impl<'a> ComponentFactory<'a> for Observed<'_, '_, 'a> {
    fn source(&self) -> &'a str {
        ComponentFactory::source(self.region)
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
        let node = ComponentFactory::interpolation(self.region, expression, span)?;
        self.sites.expressions.push((node, expression));
        Ok(node)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let node = ComponentFactory::bind(self.region, name, name_span, expression, span)?;
        self.sites.expressions.push((node, expression));
        Ok(node)
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        let sites = &mut *self.sites;
        self.region
            .element(tag, namespace, attributes, span, |region, node| {
                sites.owners.push(node);
                body.run(&mut Observed { region, sites }, node);
            })
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        let sites = &mut *self.sites;
        self.region
            .component(name, attributes, span, |region, node| {
                sites.owners.push(node);
                body.run(&mut Observed { region, sites }, node);
            })
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.region.record(record)
    }
}
