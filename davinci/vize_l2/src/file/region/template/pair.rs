//! Atomic original If/Else root pair; neither header is an empty expression.

use super::{TemplateVisibility, body};
use crate::artifact::{ElementAllocation, RegionBuilder};
use crate::file::region::FileRegion;
use crate::file::{ConditionalElseInput, FileConditionalState, TemplateBody, TemplateChildRegion};
use crate::lang::js::TemplateRouteIssueKind as Kind;
use crate::op::{Attribute, Namespace};
use core::ops::DerefMut;
use vize_l0::{Span, Vec, id::NodeId};
use vize_l1::markup::{
    ChildView, ChildrenView, ElementView, NativeTemplateComponent, SurfaceChild,
};

#[cfg(test)]
mod composite_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_tests;

pub(super) fn construct<'o, 'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: ChildView<'o, 'a>,
    tail: &mut ChildrenView<'o, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, TemplateVisibility>,
) -> Result<(NodeId, usize), Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let original = child.into_element().ok_or(Kind::InvalidEvent)?;
    let span = body::element_span(&original)?;
    // One complete first-header visit; ordinary/For consumes no tail and
    // dispatches its already prepared original header without re-observation.
    let mut header = body::header::construct_for_pair(&original, selected, region)?;
    let Some(observed) = header.conditional.take() else {
        return body::element_prepared(original, selected, span, header, region)
            .map(|node| (node, 1));
    };
    let result = (|| {
        let second = tail.next().ok_or(Kind::IncompleteChildren)?;
        if original.parent_element().is_some()
            || original.ordinal() != 0
            || selected.children().len() != 2
            || !core::ptr::eq(second.component(), selected.component())
            || second.parent_element().is_some()
            || second.ordinal() != 1
        {
            return Err(Kind::InvalidEvent);
        }
        let otherwise = second.into_element().ok_or(Kind::UnsupportedChild)?;
        observe_else(selected, &otherwise, observed.index, region)?;
        #[cfg(test)]
        tests::inject(1);
        // Both genuine headers remain parked through all Boolean/body policy.
        let else_span = body::element_span(&otherwise)?;
        let operand = region
            .facts
            .file_conditionals()
            .get(observed.index)
            .and_then(|row| row.else_input())
            .ok_or(Kind::InvalidEvent)?
            .operand();
        if span.end != else_span.start
            || operand.element_span() != else_span
            || header.for_head.is_some()
            || header.model.is_some()
            || !header.attributes.is_empty()
            || !header.handlers.is_empty()
        {
            return Err(Kind::UnsupportedChild);
        }
        region.with_walk(span, |file| {
            let node = file.mint_conditional(observed.index, span)?;
            let ready = header.resolve_handlers(file)?;
            let (first_body, first_allocation) =
                branch_body(original, selected, span, ready.attributes, file)?;
            file.attach_attribute_values(ready.values, first_body, first_allocation)?;
            #[cfg(test)]
            tests::inject(2);
            let record = file
                .facts
                .file_conditionals()
                .get(observed.index)
                .ok_or(Kind::InvalidEvent)?;
            let input = record.else_input().ok_or(Kind::InvalidEvent)?;
            file.region
                .begin_file_else(node, input.operand())
                .map_err(Kind::Artifact)?;
            #[cfg(test)]
            tests::inject(3);
            let (else_body, allocation) = branch_body(
                otherwise,
                selected,
                else_span,
                Vec::new_in(&selected.component().allocator()),
                file,
            )?;
            file.facts
                .file_conditionals_mut()
                .get_mut(observed.index)
                .and_then(|row| row.otherwise.as_deref_mut())
                .ok_or(Kind::InvalidEvent)?
                .body = Some((else_body, allocation.pointer()));
            #[cfg(test)]
            tests::inject(4);
            let pointer = file.region.finish_file_if(node).map_err(Kind::Artifact)?;
            #[cfg(test)]
            tests::inject(5);
            file.facts
                .file_conditionals_mut()
                .get_mut(observed.index)
                .ok_or(Kind::InvalidEvent)?
                .original = Some(pointer);
            Ok(node)
        })
    })();
    let record = region
        .facts
        .file_conditionals_mut()
        .get_mut(observed.index)
        .ok_or(Kind::InvalidEvent)?;
    record.state = match result {
        Ok(node) => FileConditionalState::Admitted(node),
        Err(kind) => FileConditionalState::Refused(kind),
    };
    result.map(|node| (node, 2))
}

fn observe_else<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    original: &ElementView<'_, 'a>,
    index: usize,
    region: &mut FileRegion<'_, 'b, 'a, R, TemplateVisibility>,
) -> Result<(), Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let mut invalid = false;
    let mut seen = false;
    for (ordinal, attribute) in original.attributes().enumerate() {
        if attribute.ordinal() != ordinal || !core::ptr::eq(attribute.element(), original.surface())
        {
            return Err(Kind::InvalidEvent);
        }
        let head = selected
            .observe_attribute_head(attribute.reborrow())
            .map_err(|_| Kind::UnsupportedChild)?;
        match head.observe_else() {
            Ok(operand) if !seen => {
                if operand
                    .admitted_for(selected, attribute.reborrow())
                    .is_none()
                {
                    return Err(Kind::InvalidEvent);
                }
                region
                    .facts
                    .file_conditionals_mut()
                    .get_mut(index)
                    .ok_or(Kind::InvalidEvent)?
                    .otherwise = Some(alloc::boxed::Box::new(ConditionalElseInput {
                    operand,
                    body: None,
                }));
                seen = true;
            }
            _ => invalid = true,
        }
    }
    if !seen || invalid {
        return Err(Kind::UnsupportedChild);
    }
    Ok(())
}

fn branch_body<'a: 'b, 'b, R>(
    original: ElementView<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    span: Span,
    attributes: Vec<'a, Attribute<'a>>,
    region: &mut FileRegion<'_, 'b, 'a, R, TemplateVisibility>,
) -> Result<(NodeId, ElementAllocation<'a>), Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let mut result = Err(Kind::IncompleteChildren);
    let output = region
        .file_element_body(
            original.surface().tag(),
            Namespace::Html,
            attributes,
            span,
            BranchBody {
                original,
                selected,
                result: &mut result,
            },
        )
        .map_err(Kind::Artifact)?;
    result?;
    Ok(output)
}

/// Original empty/singleton body, or one Text and one checked interpolation.
struct BranchBody<'selected, 'owner, 'result, 'a> {
    original: ElementView<'owner, 'a>,
    selected: &'selected NativeTemplateComponent<'a>,
    result: &'result mut Result<(), Kind>,
}
impl<'a> TemplateBody<'a, TemplateVisibility> for BranchBody<'_, '_, '_, 'a> {
    fn run<'r, 'b>(
        self,
        region: &mut TemplateChildRegion<'r, 'b, 'a, TemplateVisibility>,
        _: NodeId,
    ) where
        'a: 'b,
        'b: 'r,
    {
        let mut previous = None;
        let mut paired = false;
        for child in self.original.children() {
            let interpolation = match child.surface() {
                SurfaceChild::Text(_) => false,
                SurfaceChild::Interpolation(_) => true,
                _ => {
                    *self.result = Err(Kind::UnsupportedChild);
                    return;
                }
            };
            if paired || previous == Some(interpolation) {
                *self.result = Err(Kind::UnsupportedChild);
                return;
            }
            paired = previous.is_some();
            previous = Some(interpolation);
            let result = match child.surface() {
                SurfaceChild::Text(_) => body::construct(self.selected, child, &mut region.inner),
                SurfaceChild::Interpolation(_) => {
                    super::interpolation::checked_primitive(self.selected, child, &mut region.inner)
                }
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
