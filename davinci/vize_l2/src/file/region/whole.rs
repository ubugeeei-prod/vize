//! A live whole-driver borrow arms before header/source/policy operations.

use super::{TemplatePolicy, TemplateRegion};
use crate::artifact::{ArtifactError, ComponentBody, ComponentFactory};
use crate::expr::JsExpr;
use crate::file::template::TemplateWalk;
use crate::file::{FileIssueKind, TemplateIssue};
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use core::ops::{Deref, DerefMut};
use vize_l0::{Span, id::NodeId};

/// A neutral live borrow, not a native construction receipt. Its fields and
/// state cannot be supplied by a caller; completion only closes this guard.
///
/// ```compile_fail
/// use vize_l2::file::{TemplatePolicy, TemplateWalkRegion};
/// fn disarm<P: TemplatePolicy>(walk: &mut TemplateWalkRegion<'_, '_, '_, P>) {
///     walk.armed = false;
/// }
/// ```
pub struct TemplateWalkRegion<'g, 'f, 'a, P: TemplatePolicy> {
    region: &'g mut TemplateRegion<'f, 'a, P>,
    previous: TemplateWalk,
    armed: bool,
}

impl<'f, 'a, P: TemplatePolicy> TemplateRegion<'f, 'a, P> {
    /// Surround the same existing driver, including work before its first mint.
    /// This lower diagnostic API grants no original Component or Vue authority.
    pub fn walk(&mut self, span: Span) -> Result<TemplateWalkRegion<'_, 'f, 'a, P>, ArtifactError> {
        if matches!(self.inner.facts.template_walk, TemplateWalk::Pending(_)) {
            return Err(self.inner.reject(span, FileIssueKind::ActiveTemplateWalk));
        }
        if self
            .source()
            .get(span.start as usize..span.end as usize)
            .is_none()
        {
            return Err(self.inner.reject(span, FileIssueKind::InvalidSpan));
        }
        let previous = self.inner.facts.template_walk.enter(span);
        Ok(TemplateWalkRegion {
            region: self,
            previous,
            armed: true,
        })
    }
}
impl<P: TemplatePolicy> TemplateWalkRegion<'_, '_, '_, P> {
    /// Consume only after normal return from the actual surrounding driver.
    /// Sticky inner interruption cannot become Complete through this method.
    pub fn complete(mut self) -> Result<(), TemplateIssue> {
        self.region
            .inner
            .facts
            .template_walk
            .complete(self.previous);
        self.armed = false;
        match self.region.inner.facts.template_walk.interruption() {
            Some(issue) => Err(issue),
            None => Ok(()),
        }
    }
}
impl<P: TemplatePolicy> Drop for TemplateWalkRegion<'_, '_, '_, P> {
    fn drop(&mut self) {
        if self.armed {
            self.region.inner.facts.template_walk.interrupt();
        }
    }
}
// Deref lends only the already public opaque diagnostic region, never Facts,
// Builder or the private recorder. Existing concrete-child methods stay usable.
impl<'f, 'a, P: TemplatePolicy> Deref for TemplateWalkRegion<'_, 'f, 'a, P> {
    type Target = TemplateRegion<'f, 'a, P>;
    fn deref(&self) -> &Self::Target {
        self.region
    }
}
impl<P: TemplatePolicy> DerefMut for TemplateWalkRegion<'_, '_, '_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.region
    }
}
impl<'a, P: TemplatePolicy> ComponentFactory<'a> for TemplateWalkRegion<'_, '_, 'a, P> {
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
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region.element(tag, namespace, attributes, span, body)
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.region.component(name, attributes, span, body)
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.region.record(record)
    }
}
