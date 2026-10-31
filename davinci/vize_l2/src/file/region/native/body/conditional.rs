//! Same-scope Boolean root If and one original static Element callback.

use super::super::conditional::ObservedConditional;
use super::{NativeVisibility, header};
use crate::artifact::RegionBuilder;
use crate::file::build::ScopedResolution;
use crate::file::region::{FileRegion, TemplatePolicy};
use crate::file::{NativeFileConditionalState, TemplateBody, TemplateChildRegion};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use crate::op::Namespace;
use crate::resolution::Usage;
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
        let expression = file
            .facts
            .native_conditionals()
            .get(observed.index)
            .and_then(|record| record.input.expression())
            .ok_or(Kind::InvalidEvent)?;
        let table = file.resolve(expression, span).map_err(Kind::Artifact)?;
        let [occurrence] = table.occurrences() else {
            return Err(Kind::UnsupportedChild);
        };
        let declaration = file
            .facts
            .declarations
            .get(occurrence.binding.index() as usize)
            .ok_or(Kind::UnsupportedChild)?;
        if occurrence.usage != Usage::Read
            || declaration.id != occurrence.binding
            || declaration.scope != file.scope
            || !file.policy.visible(declaration)
            || declaration.boolean_initializer().is_none()
            || declaration.name.as_str() != occurrence.name
            || expression.source != occurrence.name
        {
            return Err(Kind::UnsupportedChild);
        }
        let binding = occurrence.binding;
        let record = file
            .facts
            .native_conditionals()
            .get(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        if record.scope != file.scope || !record.input.matches_expression(expression) {
            return Err(Kind::InvalidEvent);
        }
        let node = file
            .region
            .begin_native_if(&record.input, span)
            .map_err(Kind::Artifact)?;
        file.facts.expressions.insert(
            node,
            ScopedResolution {
                scope: file.scope,
                table,
            },
        );
        file.facts
            .native_conditionals_mut()
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?
            .binding = Some(binding);
        #[cfg(test)]
        super::super::conditional::tests::after_mint();
        let header = header.resolve_handlers(file)?;
        let mut body_result = Err(Kind::IncompleteChildren);
        let (body_node, allocation) = file
            .native_element_body(
                original.surface().tag(),
                Namespace::Html,
                header.attributes,
                span,
                StaticBody {
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

struct StaticBody<'selected, 'owner, 'result, 'a> {
    original: NativeElement<'owner, 'a>,
    selected: &'selected NativeTemplateComponent<'a>,
    result: &'result mut Result<(), Kind>,
}

impl<'a> TemplateBody<'a, NativeVisibility> for StaticBody<'_, '_, '_, 'a> {
    fn run<'r, 'b>(self, region: &mut TemplateChildRegion<'r, 'b, 'a, NativeVisibility>, _: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        let mut seen = false;
        for child in self.original.children() {
            if seen || !matches!(child.surface(), SurfaceChild::Text(_)) {
                *self.result = Err(Kind::UnsupportedChild);
                return;
            }
            seen = true;
            if let Err(kind) = super::construct(self.selected, child, &mut region.inner) {
                *self.result = Err(kind);
                return;
            }
        }
        *self.result = Ok(());
    }
}
