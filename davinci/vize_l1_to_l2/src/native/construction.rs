//! Private recursive construction; public diagnostics keep their original bound.

use vize_l0::{Span, id::NodeId};
use vize_l2::artifact::{ComponentBody, ComponentFactory, L2TemplateError};
use vize_l2::expr::JsExpr;
use vize_l2::op::{Attribute, Namespace};
use vize_l2::provenance::ProvenanceRecord;

/// A body called by the actual owner's private construction capability.
pub(crate) trait ConstructionBody<'a> {
    fn run<R: ConstructionFactory<'a>>(self, region: &mut R, node: NodeId);
}

/// The existing ordinary operations, kept separate from public diagnostics.
///
/// This capability alone grants no native File or control admission receipt.
/// Genuine control methods require the separately checked current event route.
pub(crate) trait ConstructionFactory<'a>: Sized {
    fn source(&self) -> &'a str;
    #[cfg(feature = "component-listener-queries")]
    fn event_reference(
        &mut self,
        _: Span,
        _: vize_l0::Vec<'a, &'a str>,
        _: Option<&'a JsExpr<'a>>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        Err(L2TemplateError::MissingEventRecorder { span })
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, L2TemplateError>;
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, L2TemplateError>;
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError>;
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError>;
    fn element<B: ConstructionBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, L2TemplateError>;
    fn component<B: ConstructionBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, L2TemplateError>;
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), L2TemplateError>;
}

/// Stack-only delegation into a caller's real public diagnostic factory.
pub(super) struct Diagnostic<'r, R> {
    region: &'r mut R,
}

impl<'r, R> Diagnostic<'r, R> {
    pub(super) fn new(region: &'r mut R) -> Self {
        Self { region }
    }
}

impl<'a, R: ComponentFactory<'a>> ConstructionFactory<'a> for Diagnostic<'_, R> {
    fn source(&self) -> &'a str {
        self.region.source()
    }

    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, L2TemplateError> {
        self.region.text(text, span)
    }

    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, L2TemplateError> {
        self.region.comment(text, span)
    }

    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        self.region.interpolation(expression, span)
    }

    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        self.region.bind(name, name_span, expression, span)
    }

    fn element<B: ConstructionBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, L2TemplateError> {
        self.region
            .element(tag, namespace, attributes, span, DiagnosticBody(body))
    }

    fn component<B: ConstructionBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, L2TemplateError> {
        self.region
            .component(name, attributes, span, DiagnosticBody(body))
    }

    fn record(&mut self, record: ProvenanceRecord) -> Result<(), L2TemplateError> {
        self.region.record(record)
    }
}

/// Adapt the private body to whatever actual child the public factory lends.
struct DiagnosticBody<B>(B);

impl<'a, B: ConstructionBody<'a>> ComponentBody<'a> for DiagnosticBody<B> {
    fn run<R: ComponentFactory<'a>>(self, region: &mut R, node: NodeId) {
        self.0.run(&mut Diagnostic::new(region), node);
    }
}
