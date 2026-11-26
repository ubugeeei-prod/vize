//! Original receiver retains only the transient actual canonical allocation.

use super::{L2IrRegion, TemplateBody, TemplateChildRegion, TemplatePolicy};
use crate::artifact::{ElementAllocation, L2TemplateError, RegionBuilder};
use crate::op::{Attribute, Namespace};
use core::{marker::PhantomData, ops::DerefMut};
use vize_l0::{Span, id::NodeId};

impl<'a: 'b, 'b, R, P> L2IrRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    pub(in crate::file::region) fn native_element_body<B: TemplateBody<'a, P>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<(NodeId, ElementAllocation<'a>), L2TemplateError> {
        self.with_walk(span, |file| {
            let Self {
                region,
                facts,
                scope,
                policy,
                source,
                ..
            } = file;
            region.native_element(tag, namespace, attributes, span, |child, node| {
                body.run(
                    &mut TemplateChildRegion {
                        inner: L2IrRegion {
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
