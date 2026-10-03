//! One authentic header parks the complete head before any collection lookup.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::FileIssueKind;
use crate::file::for_head::{PendingFor, RejectedFileFor};
use crate::file::region::{FileRegion, Lookup};
use crate::lang::js::{NativeForInput, file::native::NativeTemplateIssueKind as Kind};
use crate::resolution::{ForResolutionErrorKind, ResolutionErrorKind, resolve_for_facts};
use core::ops::DerefMut;
use vize_l0::Span;
use vize_l1::markup::{NativeAttribute, NativeTemplateComponent};

pub(super) struct ObservedFor {
    pub(super) index: usize,
}

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_for(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<ObservedFor, Kind> {
        let block = selected.component().block();
        let name_span = block
            .span_of(attribute.surface().name.text)
            .ok_or(Kind::InvalidEvent)?;
        if !core::ptr::eq(block.root_source(), self.source)
            || !core::ptr::eq(selected.component(), attribute.component())
        {
            return Err(self.for_error(name_span, FileIssueKind::InvalidSource));
        }
        let operand = match selected.observe_attribute_for_head(attribute.reborrow()) {
            Ok(operand) => operand,
            Err(error) => {
                self.facts
                    .rejected_for_heads
                    .push(RejectedFileFor::Observation(error));
                return Err(self.for_error(name_span, FileIssueKind::UnsupportedSyntax));
            }
        };
        let value_span = operand.value_span();
        let input = match NativeForInput::new(operand) {
            Ok(input) => input,
            Err(error) => {
                self.facts
                    .rejected_for_heads
                    .push(RejectedFileFor::Syntax(error));
                return Err(self.for_error(value_span, FileIssueKind::UnsupportedSyntax));
            }
        };
        let index = self.facts.pending_for_heads.len();
        self.facts.pending_for_heads.push(PendingFor {
            input: Some(input),
            resolution: None,
            enclosing: self.scope,
            span: value_span,
        });
        if self.facts.pending_for_heads[index]
            .input
            .as_ref()
            .ok_or(Kind::InvalidEvent)?
            .admitted_for(selected, attribute)
            .is_none()
        {
            return Err(self.for_error(value_span, FileIssueKind::InvalidSource));
        }
        #[cfg(test)]
        super::for_tests::after_park();
        Ok(ObservedFor { index })
    }

    pub(super) fn resolve_for(&mut self, observed: &ObservedFor) -> Result<(), Kind> {
        let pending = self
            .facts
            .pending_for_heads
            .get(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        if pending.enclosing != self.scope || pending.resolution.is_some() {
            return Err(Kind::InvalidEvent);
        }
        let input = pending.input.as_ref().ok_or(Kind::InvalidEvent)?;
        let span = pending.span;
        let result = resolve_for_facts(
            input,
            &Lookup {
                facts: self.facts,
                scope: self.scope,
                policy: self.policy,
            },
        );
        let facts = match result {
            Ok(facts) => facts,
            Err(error) => {
                let original_source = match error.part {
                    vize_l1::embed::syntax::ForHeadPart::Aliases => input
                        .operand()
                        .syntax()
                        .aliases()
                        .and_then(Result::ok)
                        .map(|aliases| aliases.source()),
                    vize_l1::embed::syntax::ForHeadPart::Collection => input
                        .operand()
                        .syntax()
                        .collection()
                        .and_then(Result::ok)
                        .map(|collection| collection.source()),
                };
                let span = original_source
                    .and_then(|source| source.authored_covering_span(error.span).ok())
                    .unwrap_or(span);
                let input = self
                    .facts
                    .pending_for_heads
                    .get_mut(observed.index)
                    .ok_or(Kind::InvalidEvent)?
                    .input
                    .take()
                    .ok_or(Kind::InvalidEvent)?;
                let kind = match error.kind {
                    ForResolutionErrorKind::Reference(ResolutionErrorKind::MissingBinding) => {
                        FileIssueKind::UnresolvedReference
                    }
                    ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan) => {
                        FileIssueKind::InvalidSpan
                    }
                    ForResolutionErrorKind::Reference(ResolutionErrorKind::TraversalLimit) => {
                        FileIssueKind::BindingLimit
                    }
                    _ => FileIssueKind::UnsupportedSyntax,
                };
                self.facts
                    .rejected_for_heads
                    .push(RejectedFileFor::Resolution {
                        input: alloc::boxed::Box::new(input),
                        error,
                    });
                return Err(self.for_error(span, kind));
            }
        };
        let pending = self
            .facts
            .pending_for_heads
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        let input = pending.input.take().ok_or(Kind::InvalidEvent)?;
        match facts.join(input) {
            Ok(resolution) => {
                pending.resolution = Some(resolution);
                Ok(())
            }
            Err(input) => {
                pending.input = Some(*input);
                Err(self.for_error(span, FileIssueKind::InvalidSource))
            }
        }
    }

    pub(super) fn for_error(&mut self, span: Span, kind: FileIssueKind) -> Kind {
        let _ = self.reject(span, kind);
        Kind::For { span, kind }
    }
}
