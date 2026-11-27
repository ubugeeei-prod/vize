//! Observe once at the original child, under the actual File's sticky guard.

use super::NativeVisibility;
use crate::ir::NativeL2IrInterpolationFailure;
use crate::ir::region::L2IrRegion;
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind as Kind};
use crate::template::RegionBuilder;
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{ChildView, SelectedTemplateComponent, SurfaceChild};

pub(in crate::ir::region::native) fn construct<'a: 'b, 'b, R>(
    selected: &SelectedTemplateComponent<'a>,
    child: ChildView<'_, 'a>,
    region: &mut L2IrRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    // Arm before preparation and the sole stock parse, including typed failures.
    region.with_walk(selected.component().block().span(), |region| {
        #[cfg(test)]
        super::interruption::before_observe();
        match selected.observe_interpolation_expression(child.reborrow()) {
            Ok(operand) => super::construct(
                selected,
                child,
                NativeInterpolationInput::from_operand(operand),
                region,
                Ok(()),
            ),
            Err(failure) => {
                let span = full_span(selected, &child);
                let kind = failure.kind();
                region
                    .facts
                    .native_interpolation_failures
                    .push(NativeL2IrInterpolationFailure { span, failure });
                #[cfg(test)]
                super::interruption::after_failure_park();
                Err(Kind::InterpolationPreparation { span, kind })
            }
        }
    })
}

fn full_span(selected: &SelectedTemplateComponent<'_>, child: &ChildView<'_, '_>) -> Span {
    let block = selected.component().block();
    if let SurfaceChild::Interpolation(interpolation) = child.surface()
        && let Some(open) = block.span_of(interpolation.open.text)
        && let Some(close) = block.span_of(interpolation.close.text)
    {
        return Span::new(open.start, close.end);
    }
    block.span()
}
