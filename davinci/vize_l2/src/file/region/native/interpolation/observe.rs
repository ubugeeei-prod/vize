//! Observe once at the original child, under the actual File's sticky guard.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::NativeFileInterpolationFailure;
use crate::file::region::FileRegion;
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind as Kind};
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};
use vize_l1::{
    SurfaceChild,
    markup::{NativeChild, NativeTemplateComponent},
};

pub(in crate::file::region::native) fn construct<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
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
                    .push(NativeFileInterpolationFailure { span, failure });
                #[cfg(test)]
                super::interruption::after_failure_park();
                Err(Kind::InterpolationPreparation { span, kind })
            }
        }
    })
}

fn full_span(selected: &NativeTemplateComponent<'_>, child: &NativeChild<'_, '_>) -> Span {
    let block = selected.component().block();
    if let SurfaceChild::Interpolation(interpolation) = child.surface()
        && let Some(open) = block.span_of(interpolation.open.text)
        && let Some(close) = block.span_of(interpolation.close.text)
    {
        return Span::new(open.start, close.end);
    }
    block.span()
}
