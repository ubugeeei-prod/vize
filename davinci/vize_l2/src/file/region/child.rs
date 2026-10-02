//! Concrete borrowed children retain the same private semantic recorder.

use super::{FileRegion, TemplatePolicy};
use crate::artifact::{ArtifactError, ComponentBody, ComponentFactory, RegionBuilder};
use crate::expr::JsExpr;
use crate::file::{FileIssueKind, TemplateIssue};
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use core::{marker::PhantomData, ops::DerefMut};
use vize_l0::{Span, id::NodeId};

/// A child minted by the real file recorder, with no raw builder extraction.
///
/// ```compile_fail
/// use vize_l2::file::{TemplateChildRegion, TemplatePolicy};
/// fn extract<'r, 'b, 'a, P: TemplatePolicy>(child: &mut TemplateChildRegion<'r, 'b, 'a, P>) {
///     let _ = &mut child.inner;
/// }
/// ```
pub struct TemplateChildRegion<'r, 'b, 'a, P: TemplatePolicy> {
    pub(super) inner: FileRegion<'r, 'b, 'a, &'r mut RegionBuilder<'b, 'a>, P>,
}

/// A concrete callback preserves compile-local captures without a GAT.
/// An ordinary public body cannot strengthen its child factory bound.
///
/// ```compile_fail
/// use vize_l2::artifact::ComponentFactory;
/// use vize_l2::file::{TemplateBody, TemplatePolicy};
/// use vize_l2::op::{Attribute, Namespace};
/// fn stronger<'a, R: ComponentFactory<'a>, P: TemplatePolicy, B: TemplateBody<'a, P>>(
///     child: &mut R, attributes: vize_l0::Vec<'a, Attribute<'a>>, span: vize_l0::Span, body: B,
/// ) {
///     let _ = child.element_with("p", Namespace::Html, attributes, span, body);
/// }
/// ```
pub trait TemplateBody<'a, P: TemplatePolicy> {
    fn run<'r, 'b>(self, child: &mut TemplateChildRegion<'r, 'b, 'a, P>, node: NodeId)
    where
        'a: 'b,
        'b: 'r;
}

impl<'a: 'b, 'b, R, P> FileRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    pub(super) fn element_body<B: TemplateBody<'a, P>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |file| {
            let Self {
                region,
                facts,
                scope,
                policy,
                source,
                ..
            } = file;
            region.element(tag, namespace, attributes, span, |child, node| {
                body.run(
                    &mut TemplateChildRegion {
                        inner: FileRegion {
                            region: child,
                            facts,
                            scope: *scope,
                            policy: *policy,
                            source,
                            borrow: PhantomData,
                        },
                    },
                    node,
                );
            })
        })
    }

    pub(super) fn component_body<B: TemplateBody<'a, P>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |file| {
            let Self {
                region,
                facts,
                scope,
                policy,
                source,
                ..
            } = file;
            region.component(name, attributes, span, |child, node| {
                facts.template_issues.push(TemplateIssue {
                    node: Some(node),
                    span,
                    kind: FileIssueKind::UnsupportedComponent,
                });
                body.run(
                    &mut TemplateChildRegion {
                        inner: FileRegion {
                            region: child,
                            facts,
                            scope: *scope,
                            policy: *policy,
                            source,
                            borrow: PhantomData,
                        },
                    },
                    node,
                );
            })
        })
    }
}

impl<'f, 'a, P: TemplatePolicy> super::TemplateRegion<'f, 'a, P> {
    pub fn element_with<B: TemplateBody<'a, P>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner
            .element_body(tag, namespace, attributes, span, body)
    }
    pub fn component_with<B: TemplateBody<'a, P>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.component_body(name, attributes, span, body)
    }
}

impl<'a: 'b, 'b, 'r, P: TemplatePolicy> TemplateChildRegion<'r, 'b, 'a, P> {
    pub fn element_with<B: TemplateBody<'a, P>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner
            .element_body(tag, namespace, attributes, span, body)
    }
    pub fn component_with<B: TemplateBody<'a, P>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.component_body(name, attributes, span, body)
    }
}

impl<'a: 'b, 'b, P: TemplatePolicy> ComponentFactory<'a> for TemplateChildRegion<'_, 'b, 'a, P> {
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
