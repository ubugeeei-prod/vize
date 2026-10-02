//! Monomorphized construction capabilities; callbacks receive real child factories.

use super::{ArtifactError, RegionBuilder};
use crate::expr::{ExprRef, JsExpr};
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use vize_l0::{Span, id::NodeId};

/// A recursive body, invoked only by its factory's actual minted owner.
///
/// A concrete callback avoids requiring captured compile-local source and
/// recorder borrows to outlive every possible child-factory lifetime.
pub trait ComponentBody<'a> {
    fn run<R: ComponentFactory<'a>>(self, region: &mut R, node: NodeId);
}

/// A checked Component construction route, with retained JavaScript payloads.
///
/// Implementing this diagnostic capability does not confer file-resolution
/// completeness. An authoritative file owner keeps its recorder private and
/// resolves, constructs and associates each expression inside its own methods.
pub trait ComponentFactory<'a>: Sized {
    fn source(&self) -> &'a str;
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError>;
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError>;
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError>;
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError>;
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError>;
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError>;
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError>;
}

impl<'a> ComponentFactory<'a> for RegionBuilder<'_, 'a> {
    fn source(&self) -> &'a str {
        self.builder.parts.source
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        RegionBuilder::text(self, text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        RegionBuilder::comment(self, text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        RegionBuilder::interpolation(self, ExprRef::Js(expression), span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        RegionBuilder::bind(self, name, name_span, ExprRef::Js(expression), span)
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        RegionBuilder::element(self, tag, namespace, attributes, span, |child, node| {
            body.run(child, node);
        })
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        RegionBuilder::component(self, name, attributes, span, |child, node| {
            body.run(child, node);
        })
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        RegionBuilder::record(self, record)
    }
}
