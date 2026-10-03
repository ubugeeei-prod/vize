//! Park, authenticate, resolve and mint at one real original root event.

use super::NativeVisibility;
use crate::artifact::ComponentFactory;
use crate::file::region::RootRegion;
use crate::file::{NativeFileInterpolation, NativeFileInterpolationState};
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind as Kind};
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeChild, NativeTemplateComponent};

#[cfg(test)]
mod interruption;

pub(super) fn construct<'a>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    input: NativeInterpolationInput<'a>,
    region: &mut RootRegion<'_, 'a, NativeVisibility>,
    check: Result<(), Kind>,
) -> Result<NodeId, Kind> {
    let index = region.facts.native_interpolations.len();
    let span = input.operand().full_span();
    region
        .facts
        .native_interpolations
        .push(NativeFileInterpolation {
            input,
            state: NativeFileInterpolationState::Pending,
        });
    #[cfg(test)]
    interruption::after_park();
    let result = (|| {
        check?;
        let expression = region
            .facts
            .native_interpolations
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?
            .input
            .admitted_for(selected, child)
            .map_err(Kind::Interpolation)?
            .expression();
        // This actual File method owns the sole resolver and attached Op mint.
        let node = region
            .interpolation(expression, span)
            .map_err(Kind::Artifact)?;
        #[cfg(test)]
        interruption::after_mint();
        region.facts.native_interpolation_nodes.insert(node, index);
        Ok(node)
    })();
    let record = region
        .facts
        .native_interpolations
        .get_mut(index)
        .ok_or(Kind::InvalidEvent)?;
    record.state = match result {
        Ok(node) => NativeFileInterpolationState::Admitted(node),
        Err(kind) => NativeFileInterpolationState::Refused(kind),
    };
    result
}

#[cfg(test)]
mod tests;
