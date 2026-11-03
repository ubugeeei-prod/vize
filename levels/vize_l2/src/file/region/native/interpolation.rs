//! Park, authenticate, resolve and mint at one real original child event.

use super::NativeVisibility;
use crate::artifact::{ComponentFactory, RegionBuilder};
use crate::file::region::{FileRegion, TemplatePolicy};
use crate::file::{NativeFileInterpolation, NativeFileInterpolationState};
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind as Kind};
use crate::resolution::Usage;
use core::ops::DerefMut;
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeChild, NativeTemplateComponent};

mod observe;
pub(super) use observe::construct as observe;

#[cfg(test)]
mod interruption;

pub(super) fn construct<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    input: NativeInterpolationInput<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
    check: Result<(), Kind>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
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

/// Reuse the original receiver's normal owner/resolver/mint. Its typed source
/// result stays parked even if this narrower whole-If profile rejects it.
pub(super) fn checked_primitive<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let index = region.facts.native_interpolations.len();
    let node = observe(selected, child, region)?;
    let result = (|| {
        let [record] = region
            .facts
            .native_interpolations
            .get(index..)
            .ok_or(Kind::InvalidEvent)?
        else {
            return Err(Kind::InvalidEvent);
        };
        let scoped = region
            .facts
            .expressions
            .get(node)
            .ok_or(Kind::InvalidEvent)?;
        let expression = scoped.table.expression();
        let operand = record.input().operand();
        let syntax = operand.syntax();
        let source = syntax.source();
        let identifier = expression
            .ast
            .get_identifier_reference()
            .ok_or(Kind::UnsupportedChild)?;
        let [occurrence] = scoped.table.occurrences() else {
            return Err(Kind::UnsupportedChild);
        };
        let declaration = region
            .facts
            .declarations
            .get(occurrence.binding.index() as usize)
            .ok_or(Kind::UnsupportedChild)?;
        if record.state() != NativeFileInterpolationState::Admitted(node)
            || region.facts.native_interpolation_nodes.get(node) != Some(&index)
            || scoped.scope != region.scope
            || !core::ptr::eq(source.authored_root(), region.source)
            || !core::ptr::eq(
                source.authored_root(),
                selected.component().block().root_source(),
            )
            || !core::ptr::eq(
                syntax.expression().ok_or(Kind::InvalidEvent)?,
                expression.ast,
            )
            || !core::ptr::eq(source.text(), expression.source)
            || source.span() != expression.span
            || expression.coordinates.is_none()
            || source.decode_map().is_some()
            || syntax.comments().next().is_some()
            || expression.source != identifier.name.as_str()
            || occurrence.name != identifier.name.as_str()
            || occurrence.usage != Usage::Read
            || expression.ast_span_to_source(identifier.span) != Some(occurrence.span)
            || expression.authored_span(occurrence.span) != Some(source.span())
            || declaration.id != occurrence.binding
            || declaration.scope != region.scope
            || declaration.name.as_str() != occurrence.name
            || !declaration.initializer.is_primitive()
            || !region.policy.visible(declaration)
        {
            return Err(Kind::UnsupportedChild);
        }
        Ok(node)
    })();
    if let Err(kind) = result {
        region
            .facts
            .native_interpolations
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?
            .state = NativeFileInterpolationState::Refused(kind);
    }
    result
}

#[cfg(test)]
mod tests;
