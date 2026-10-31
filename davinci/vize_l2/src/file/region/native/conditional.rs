//! Park a whole conditional only at its real selected header event.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::region::FileRegion;
use crate::file::{NativeFileConditional, NativeFileConditionalState};
use crate::lang::js::{NativeConditionalInput, NativeTemplateIssueKind as Kind};
use core::ops::DerefMut;
use vize_l1::markup::{NativeAttributeHead, NativeConditionKind, NativeTemplateComponent};

pub(super) struct ObservedConditional {
    pub(super) index: usize,
}

#[cfg(test)]
pub(super) mod tests;

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_conditional(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        head: &NativeAttributeHead<'_, 'a>,
    ) -> Result<ObservedConditional, Kind> {
        if head.condition_kind() != Some(NativeConditionKind::If)
            || !core::ptr::eq(selected.component(), head.selected().component())
            || !core::ptr::eq(self.source, selected.component().block().root_source())
        {
            return Err(Kind::InvalidEvent);
        }
        #[cfg(test)]
        tests::before_observe();
        let operand = match head.observe_expression() {
            Ok(operand) => operand,
            Err(failure) => {
                self.facts.native_conditional_failures_mut().push(failure);
                #[cfg(test)]
                tests::after_failure_park();
                return Err(Kind::UnsupportedChild);
            }
        };
        let index = self.facts.native_conditionals().len();
        self.facts
            .native_conditionals_mut()
            .push(NativeFileConditional {
                input: NativeConditionalInput::from_operand(operand),
                state: NativeFileConditionalState::Pending,
                scope: self.scope,
                binding: None,
                original: None,
            });
        #[cfg(test)]
        tests::after_park();
        let record = self
            .facts
            .native_conditionals_mut()
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        if record
            .input
            .admitted_for(selected, head.attribute().reborrow())
            .is_err()
        {
            record.state = NativeFileConditionalState::Refused(Kind::UnsupportedChild);
            return Err(Kind::UnsupportedChild);
        }
        Ok(ObservedConditional { index })
    }
}
