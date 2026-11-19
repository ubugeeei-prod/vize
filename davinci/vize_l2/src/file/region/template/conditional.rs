//! Park a whole conditional only at its real selected header event.

use super::TemplateVisibility;
use crate::artifact::RegionBuilder;
use crate::file::build::ScopedResolution;
use crate::file::region::{FileRegion, TemplatePolicy};
use crate::file::{FileConditional, FileConditionalState};
use crate::lang::js::{ConditionalInput, TemplateRouteIssueKind as Kind};
use crate::resolution::Usage;
use core::ops::DerefMut;
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{AttributeHead, ConditionKind, NativeTemplateComponent};

pub(super) struct ObservedConditional {
    pub(super) index: usize,
}

#[cfg(test)]
pub(super) mod tests;

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, TemplateVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_conditional(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        head: &AttributeHead<'_, 'a>,
    ) -> Result<ObservedConditional, Kind> {
        if head.condition_kind() != Some(ConditionKind::If)
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
                self.facts.file_conditional_failures_mut().push(failure);
                #[cfg(test)]
                tests::after_failure_park();
                return Err(Kind::UnsupportedChild);
            }
        };
        let index = self.facts.file_conditionals().len();
        self.facts
            .file_conditionals_mut()
            .push(FileConditional {
                input: ConditionalInput::from_operand(operand),
                state: FileConditionalState::Pending,
                scope: self.scope,
                binding: None,
                original: None,
                otherwise: None,
            });
        #[cfg(test)]
        tests::after_park();
        let record = self
            .facts
            .file_conditionals_mut()
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        if record
            .input
            .admitted_for(selected, head.attribute().reborrow())
            .is_err()
        {
            record.state = FileConditionalState::Refused(Kind::UnsupportedChild);
            return Err(Kind::UnsupportedChild);
        }
        Ok(ObservedConditional { index })
    }

    /// The same original Boolean proof and real If mint for both root shapes.
    pub(super) fn mint_conditional(&mut self, index: usize, span: Span) -> Result<NodeId, Kind> {
        let expression = self
            .facts
            .file_conditionals()
            .get(index)
            .and_then(|record| record.input.expression())
            .ok_or(Kind::InvalidEvent)?;
        let table = self.resolve(expression, span).map_err(Kind::Artifact)?;
        let [occurrence] = table.occurrences() else {
            return Err(Kind::UnsupportedChild);
        };
        let declaration = self
            .facts
            .declarations
            .get(occurrence.binding.index() as usize)
            .ok_or(Kind::UnsupportedChild)?;
        if occurrence.usage != Usage::Read
            || declaration.id != occurrence.binding
            || declaration.scope != self.scope
            || !self.policy.visible(declaration)
            || declaration.boolean_initializer().is_none()
            || declaration.name.as_str() != occurrence.name
            || expression.source != occurrence.name
        {
            return Err(Kind::UnsupportedChild);
        }
        let binding = occurrence.binding;
        let record = self
            .facts
            .file_conditionals()
            .get(index)
            .ok_or(Kind::InvalidEvent)?;
        if record.scope != self.scope || !record.input.matches_expression(expression) {
            return Err(Kind::InvalidEvent);
        }
        let node = self
            .region
            .begin_file_if(&record.input, span)
            .map_err(Kind::Artifact)?;
        self.facts.expressions.insert(
            node,
            ScopedResolution {
                scope: self.scope,
                table,
            },
        );
        self.facts
            .file_conditionals_mut()
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?
            .binding = Some(binding);
        #[cfg(test)]
        tests::after_mint();
        Ok(node)
    }
}
