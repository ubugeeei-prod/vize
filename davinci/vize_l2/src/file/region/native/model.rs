//! Original Model custody and one semantic read/write of its mutable binding.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::build::ScopedResolution;
use crate::file::region::{FileRegion, Lookup, TemplatePolicy};
use crate::file::{
    DeclarationKind, FileIssueKind, Namespace, NativeFileModel, NativeFileModelState,
};
use crate::lang::js::{NativeModelInput, NativeTemplateIssueKind as Kind};
use crate::resolution::{ResolutionErrorKind, Usage, resolve_model_facts};
use core::ops::DerefMut;
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeAttributeHead, NativeTemplateComponent};

pub(super) struct ObservedModel {
    pub(super) index: usize,
}

#[cfg(test)]
pub(super) mod tests;

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_model(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        head: &NativeAttributeHead<'_, 'a>,
    ) -> Result<ObservedModel, Kind> {
        if !core::ptr::eq(selected.component(), head.selected().component())
            || !core::ptr::eq(self.source, selected.component().block().root_source())
        {
            return Err(Kind::InvalidEvent);
        }
        let model = head
            .model()
            .map_err(|_| Kind::UnsupportedChild)?
            .ok_or(Kind::InvalidEvent)?;
        #[cfg(test)]
        tests::before_observe();
        let operand = match model.observe_expression() {
            Ok(operand) => operand,
            Err(failure) => {
                self.facts.native_model_failures_mut().push(failure);
                #[cfg(test)]
                tests::after_failure_park();
                return Err(Kind::UnsupportedChild);
            }
        };
        let index = self.facts.native_models().len();
        self.facts.native_models_mut().push(NativeFileModel {
            input: NativeModelInput::from_operand(operand),
            state: NativeFileModelState::Pending,
            scope: self.scope,
            binding: None,
            original: None,
            element: None,
        });
        #[cfg(test)]
        tests::after_park();
        let record = self
            .facts
            .native_models_mut()
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        if record
            .input
            .admitted_for(selected, head.attribute().reborrow())
            .is_err()
        {
            record.state = NativeFileModelState::Refused(Kind::UnsupportedChild);
            return Err(Kind::UnsupportedChild);
        }
        Ok(ObservedModel { index })
    }

    /// A genuine mutable declaration grants the write, never an initializer tag.
    /// Target-specific primitive value policy remains a separate L3 decision.
    pub(super) fn attach_model(&mut self, index: usize) -> Result<NodeId, Kind> {
        let record = self
            .facts
            .native_models()
            .get(index)
            .ok_or(Kind::InvalidEvent)?;
        let expression = record.input.expression().ok_or(Kind::InvalidEvent)?;
        let span = record.input.attribute_span().ok_or(Kind::InvalidEvent)?;
        let resolved = resolve_model_facts(
            &record.input,
            &Lookup {
                facts: self.facts,
                scope: self.scope,
                policy: self.policy,
            },
        );
        let table = resolved.map_err(|error| {
            Kind::Artifact(self.reject(
                expression.authored_span(error.span).unwrap_or(span),
                match error.kind {
                    ResolutionErrorKind::MissingBinding => FileIssueKind::UnresolvedReference,
                    ResolutionErrorKind::InvalidSpan => FileIssueKind::InvalidSpan,
                    ResolutionErrorKind::TraversalLimit => FileIssueKind::BindingLimit,
                    ResolutionErrorKind::UnsupportedSyntax => FileIssueKind::UnsupportedSyntax,
                },
            ))
        })?;
        let [occurrence] = table.occurrences() else {
            return Err(Kind::UnsupportedChild);
        };
        let declaration = self
            .facts
            .declarations
            .get(occurrence.binding.index() as usize)
            .ok_or(Kind::UnsupportedChild)?;
        if occurrence.usage != Usage::ReadWrite
            || occurrence.shorthand
            || occurrence.constructor
            || declaration.id != occurrence.binding
            || declaration.scope != self.scope
            || declaration.namespace != Namespace::Value
            || !declaration.is_direct_program()
            || !matches!(
                declaration.kind,
                DeclarationKind::Let | DeclarationKind::Var
            )
            || !self.policy.visible(declaration)
            || declaration.name.as_str() != occurrence.name
            || expression.source != occurrence.name
        {
            return Err(Kind::UnsupportedChild);
        }
        let binding = occurrence.binding;
        let record = self
            .facts
            .native_models()
            .get(index)
            .ok_or(Kind::InvalidEvent)?;
        if record.scope != self.scope || !record.input.matches_expression(expression) {
            return Err(Kind::InvalidEvent);
        }
        let (node, pointer) = self
            .region
            .native_model(&record.input)
            .map_err(Kind::Artifact)?;
        self.facts.expressions.insert(
            node,
            ScopedResolution {
                scope: self.scope,
                table,
            },
        );
        let record = self
            .facts
            .native_models_mut()
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        record.binding = Some(binding);
        record.original = Some(pointer);
        #[cfg(test)]
        tests::after_mint();
        Ok(node)
    }
}
