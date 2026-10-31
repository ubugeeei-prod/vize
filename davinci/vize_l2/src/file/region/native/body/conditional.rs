//! Same-scope Boolean root If and one original bounded Element callback.

use super::super::conditional::ObservedConditional;
use super::{NativeVisibility, header};
use crate::artifact::RegionBuilder;
use crate::file::region::FileRegion;
use crate::file::{NativeFileConditionalState, TemplateBody, TemplateChildRegion};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use crate::op::Namespace;
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{NativeElement, NativeTemplateComponent, SurfaceChild};

pub(super) fn construct<'a: 'b, 'b, R>(
    original: NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    span: Span,
    header: header::Header<'a>,
    observed: ObservedConditional,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let result = region.with_walk(span, |file| {
        if original.parent_element().is_some()
            || original.ordinal() != 0
            || selected.children().len() != 1
            || header.for_head.is_some()
            || !header.attributes.is_empty()
            || !header.handlers.is_empty()
        {
            return Err(Kind::UnsupportedChild);
        }
        let node = file.mint_conditional(observed.index, span)?;
        let header = header.resolve_handlers(file)?;
        let mut body_result = Err(Kind::IncompleteChildren);
        let (body_node, allocation) = file
            .native_element_body(
                original.surface().tag(),
                Namespace::Html,
                header.attributes,
                span,
                ConditionalBody {
                    original,
                    selected,
                    result: &mut body_result,
                },
            )
            .map_err(Kind::Artifact)?;
        body_result?;
        #[cfg(test)]
        super::super::conditional::tests::after_body();
        // There are no value attributes in this single-condition header.
        // Consume the ordinary prepared storage through the normal attachment.
        file.attach_attribute_values(header.values, body_node, allocation)?;
        let pointer = file.region.finish_native_if(node).map_err(Kind::Artifact)?;
        #[cfg(test)]
        super::super::conditional::tests::after_close();
        let record = file
            .facts
            .native_conditionals_mut()
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        record.original = Some(pointer);
        Ok(node)
    });
    let record = region
        .facts
        .native_conditionals_mut()
        .get_mut(observed.index)
        .ok_or(Kind::InvalidEvent)?;
    record.state = match result {
        Ok(node) => NativeFileConditionalState::Admitted(node),
        Err(kind) => NativeFileConditionalState::Refused(kind),
    };
    result
}

struct ConditionalBody<'selected, 'owner, 'result, 'a> {
    original: NativeElement<'owner, 'a>,
    selected: &'selected NativeTemplateComponent<'a>,
    result: &'result mut Result<(), Kind>,
}

impl<'a> TemplateBody<'a, NativeVisibility> for ConditionalBody<'_, '_, '_, 'a> {
    fn run<'r, 'b>(self, region: &mut TemplateChildRegion<'r, 'b, 'a, NativeVisibility>, _: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        let mut seen = false;
        for child in self.original.children() {
            if seen {
                *self.result = Err(Kind::UnsupportedChild);
                return;
            }
            seen = true;
            let result = match child.surface() {
                SurfaceChild::Text(_) => super::construct(self.selected, child, &mut region.inner),
                SurfaceChild::Interpolation(_) => super::super::interpolation::checked_primitive(
                    self.selected,
                    child,
                    &mut region.inner,
                ),
                _ => Err(Kind::UnsupportedChild),
            };
            if let Err(kind) = result {
                *self.result = Err(kind);
                return;
            }
        }
        *self.result = Ok(());
    }
}
