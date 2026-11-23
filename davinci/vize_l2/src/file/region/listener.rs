//! Resolve a named method reference in the same lexical scope that attaches the On.
use super::{FileRegion, ScopedResolution, TemplateChildRegion, TemplatePolicy, TemplateRegion};
use crate::artifact::{ArtifactError, RegionBuilder};
use crate::expr::JsExpr;
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};

impl<'a: 'b, 'b, R, P> FileRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    fn event_reference(
        &mut self,
        name: Span,
        modifiers: vize_l0::Vec<'a, &'a str>,
        handler: Option<&'a JsExpr<'a>>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |region| {
            let table = handler.map(|js| region.resolve(js, span)).transpose()?;
            let node = region
                .region
                .attach_event_reference(name, modifiers, handler, span)?;
            if let Some(table) = table {
                region.facts.expressions.insert(
                    node,
                    ScopedResolution {
                        scope: region.scope,
                        table,
                        #[cfg(feature = "template-operand-queries")]
                        kind: crate::query::TemplateOperandKind::EventReference,
                    },
                );
            }
            Ok(node)
        })
    }
}

impl<'a, P: TemplatePolicy> TemplateRegion<'_, 'a, P> {
    /// Attach the original event and resolve its optional identifier in the current scope.
    pub fn event_reference(
        &mut self,
        name: Span,
        modifiers: vize_l0::Vec<'a, &'a str>,
        handler: Option<&'a JsExpr<'a>>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.event_reference(name, modifiers, handler, span)
    }
}

impl<'a: 'b, 'b, P: TemplatePolicy> TemplateChildRegion<'_, 'b, 'a, P> {
    /// Attach the original event inside this minted element or component.
    pub fn event_reference(
        &mut self,
        name: Span,
        modifiers: vize_l0::Vec<'a, &'a str>,
        handler: Option<&'a JsExpr<'a>>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.event_reference(name, modifiers, handler, span)
    }
}
