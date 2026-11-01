//! One original root input, its real Model callback and normally closed owner.

use super::super::model::ObservedModel;
use super::{NativeVisibility, header};
use crate::artifact::RegionBuilder;
use crate::file::region::FileRegion;
use crate::file::{NativeFileModelState, TemplateBody, TemplateChildRegion};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use crate::op::Namespace;
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{ElementClose, NativeElement, NativeTemplateComponent};

pub(super) fn construct<'a: 'b, 'b, R>(
    original: NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    span: Span,
    header: header::Header<'a>,
    observed: ObservedModel,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let result = region.with_walk(span, |file| {
        if original.parent_element().is_some()
            || original.ordinal() != 0
            || selected.children().len() != 1
            || original.surface().tag() != "input"
            || !matches!(&original.surface().close, ElementClose::NotExpected)
            || original.children().len() != 0
            || header.for_head.is_some()
            || header.conditional.is_some()
            || !header.attributes.is_empty()
            || !header.handlers.is_empty()
        {
            return Err(Kind::UnsupportedChild);
        }
        let header = header.resolve_handlers(file)?;
        let mut model_result = Err(Kind::IncompleteChildren);
        let (element, allocation) = file
            .native_element_body(
                original.surface().tag(),
                Namespace::Html,
                header.attributes,
                span,
                ModelBody {
                    index: observed.index,
                    result: &mut model_result,
                },
            )
            .map_err(Kind::Artifact)?;
        let model = model_result?;
        let pointer = allocation.pointer();
        file.attach_attribute_values(header.values, element, allocation)?;
        let record = file
            .facts
            .native_models_mut()
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        record.element = Some((element, pointer));
        record.state = NativeFileModelState::Admitted(model);
        #[cfg(test)]
        super::super::model::tests::after_close();
        Ok(element)
    });
    if let Err(kind) = result {
        region
            .facts
            .native_models_mut()
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?
            .state = NativeFileModelState::Refused(kind);
    }
    result
}

struct ModelBody<'result> {
    index: usize,
    result: &'result mut Result<NodeId, Kind>,
}

impl<'a> TemplateBody<'a, NativeVisibility> for ModelBody<'_> {
    fn run<'r, 'b>(self, region: &mut TemplateChildRegion<'r, 'b, 'a, NativeVisibility>, _: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        *self.result = region.inner.attach_model(self.index);
        #[cfg(test)]
        if self.result.is_ok() {
            super::super::model::tests::after_body();
        }
    }
}
