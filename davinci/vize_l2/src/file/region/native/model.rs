//! Original Model custody and one semantic read/write of its mutable binding.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::build::ScopedResolution;
use crate::file::region::{L2IrRegion, Lookup, TemplatePolicy};
use crate::file::{
    DeclarationKind, L2IrIssueKind, Namespace, NativeL2IrModel, NativeL2IrModelKind,
    NativeL2IrModelState,
};
use crate::lang::js::{NativeModelInput, NativeTemplateIssueKind as Kind};
use crate::resolution::{ResolutionErrorKind, Usage, resolve_model_facts};
use core::ops::DerefMut;
use vize_l0::id::NodeId;
use vize_l1::markup::{AttributeHead, SelectedTemplateComponent};

pub(super) struct ObservedModel {
    pub(super) index: usize,
}

#[cfg(test)]
pub(super) mod tests;

impl<'a: 'b, 'b, R> L2IrRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_model(
        &mut self,
        selected: &SelectedTemplateComponent<'a>,
        head: &AttributeHead<'_, 'a>,
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
        self.facts.native_models_mut().push(NativeL2IrModel {
            input: NativeModelInput::from_operand(operand),
            state: NativeL2IrModelState::Pending,
            scope: self.scope,
            binding: None,
            original: None,
            element: None,
            kind: None,
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
            record.state = NativeL2IrModelState::Refused(Kind::UnsupportedChild);
            return Err(Kind::UnsupportedChild);
        }
        Ok(ObservedModel { index })
    }

    /// A genuine mutable declaration grants the write, never an initializer tag.
    /// Target-specific primitive value policy remains a separate L3 decision.
    pub(super) fn attach_model(
        &mut self,
        index: usize,
        kind: NativeL2IrModelKind,
    ) -> Result<NodeId, Kind> {
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
                    ResolutionErrorKind::MissingBinding => L2IrIssueKind::UnresolvedReference,
                    ResolutionErrorKind::InvalidSpan => L2IrIssueKind::InvalidSpan,
                    ResolutionErrorKind::TraversalLimit => L2IrIssueKind::BindingLimit,
                    ResolutionErrorKind::UnsupportedSyntax => L2IrIssueKind::UnsupportedSyntax,
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
            .native_model(&record.input, kind)
            .map_err(Kind::Artifact)?;
        self.facts.expressions.insert(
            node,
            ScopedResolution {
                scope: self.scope,
                table,
                #[cfg(feature = "template-operand-queries")]
                kind: crate::query::TemplateOperandKind::Model,
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
