//! The live original header resolves privately, then attaches before children.

use super::NativeVisibility;
use crate::artifact::RegionBuilder;
use crate::file::handler::{HandlerRecord, PendingHandler};
use crate::file::region::{FileRegion, Lookup};
use crate::file::{FileIssueKind, RejectedFileHandler};
use crate::lang::js::{NativeHandlerInput, file::native::NativeTemplateIssueKind as Kind};
use crate::resolution::{ResolutionErrorKind, resolve_handler_facts};
use core::ops::DerefMut;
use vize_l0::Span;
use vize_l1::markup::{NativeAttribute, NativeTemplateComponent};

pub(super) struct ObservedHandler<'a> {
    pub(super) index: usize,
    pub(super) name: &'a str,
}

pub(super) struct PreparedHandler {
    pub(super) index: usize,
}

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    pub(super) fn observe_handler(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<ObservedHandler<'a>, Kind> {
        let block = selected.component().block();
        let name_span = block
            .span_of(attribute.surface().name.text)
            .ok_or(Kind::InvalidEvent)?;
        if !core::ptr::eq(block.root_source(), self.source)
            || !core::ptr::eq(selected.component(), attribute.component())
        {
            return Err(self.handler_error(name_span, FileIssueKind::InvalidSource));
        }
        let operand = match selected.observe_attribute_handler(attribute.reborrow()) {
            Ok(operand) => operand,
            Err(error) => {
                self.facts
                    .rejected_handlers
                    .push(RejectedFileHandler::Observation(error));
                return Err(self.handler_error(name_span, FileIssueKind::UnsupportedSyntax));
            }
        };
        let value_span = operand.value_span();
        let end = attribute
            .surface()
            .value
            .as_ref()
            .and_then(|value| {
                value
                    .close_quote
                    .as_ref()
                    .map_or(Some(&value.content), Some)
            })
            .and_then(|token| block.span_of(token.text))
            .ok_or(Kind::InvalidEvent)?
            .end;
        let span = Span::new(name_span.start, end);
        let input = match NativeHandlerInput::new(operand) {
            Ok(input) => input,
            Err(error) => {
                self.facts
                    .rejected_handlers
                    .push(RejectedFileHandler::Syntax(error));
                return Err(self.handler_error(value_span, FileIssueKind::UnsupportedSyntax));
            }
        };
        let index = self.facts.pending_handlers.len();
        self.facts.pending_handlers.push(PendingHandler {
            input: Some(input),
            resolution: None,
            scope: self.scope,
            span,
        });
        // Same original token, complete body, source/profile and owner. Even
        // an unexpected identity refusal retains this normally owned input.
        if self
            .facts
            .pending_handlers
            .get(index)
            .ok_or(Kind::InvalidEvent)?
            .input
            .as_ref()
            .ok_or(Kind::InvalidEvent)?
            .admitted_for(selected, attribute)
            .is_none()
        {
            return Err(self.handler_error(value_span, FileIssueKind::InvalidSource));
        }
        #[cfg(test)]
        super::interruption::after_park();
        let name = self.facts.pending_handlers[index]
            .input
            .as_ref()
            .ok_or(Kind::InvalidEvent)?
            .operand()
            .argument();
        Ok(ObservedHandler { index, name })
    }

    pub(super) fn resolve_handler(
        &mut self,
        observed: &ObservedHandler<'a>,
    ) -> Result<PreparedHandler, Kind> {
        let index = observed.index;
        let pending = self
            .facts
            .pending_handlers
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        if pending.resolution.is_some() {
            return Err(Kind::InvalidEvent);
        }
        pending.scope = self.scope;
        let value_span = pending
            .input
            .as_ref()
            .ok_or(Kind::InvalidEvent)?
            .operand()
            .value_span();
        // Keep the entire normal owner in File facts before the fallible walk.
        // A caught unwind leaves that same input parked, never dropped locally.
        let input = self
            .facts
            .pending_handlers
            .get(index)
            .ok_or(Kind::InvalidEvent)?
            .input
            .as_ref()
            .ok_or(Kind::InvalidEvent)?;
        let result = resolve_handler_facts(
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
                let input = self
                    .facts
                    .pending_handlers
                    .get_mut(index)
                    .ok_or(Kind::InvalidEvent)?
                    .input
                    .take()
                    .ok_or(Kind::InvalidEvent)?;
                let span = input
                    .operand()
                    .syntax()
                    .source()
                    .authored_covering_span(error.span)
                    .unwrap_or(value_span);
                let kind = match error.kind {
                    ResolutionErrorKind::MissingBinding => FileIssueKind::UnresolvedReference,
                    ResolutionErrorKind::InvalidSpan => FileIssueKind::InvalidSpan,
                    ResolutionErrorKind::TraversalLimit => FileIssueKind::BindingLimit,
                    ResolutionErrorKind::UnsupportedSyntax => FileIssueKind::UnsupportedSyntax,
                };
                self.facts
                    .rejected_handlers
                    .push(RejectedFileHandler::Resolution {
                        input: alloc::boxed::Box::new(input),
                        error,
                    });
                return Err(self.handler_error(span, kind));
            }
        };
        let pending = self
            .facts
            .pending_handlers
            .get_mut(index)
            .ok_or(Kind::InvalidEvent)?;
        let Some(resolution) = facts.join(&mut pending.input) else {
            return Err(self.handler_error(value_span, FileIssueKind::InvalidSource));
        };
        pending.resolution = Some(resolution);
        Ok(PreparedHandler { index })
    }

    pub(super) fn prepared_handler(
        &self,
        observed: ObservedHandler<'a>,
    ) -> Result<PreparedHandler, Kind> {
        let pending = self
            .facts
            .pending_handlers
            .get(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        let resolution = pending.resolution.as_ref().ok_or(Kind::InvalidEvent)?;
        if pending.scope != self.scope || resolution.input().operand().argument() != observed.name {
            return Err(Kind::InvalidEvent);
        }
        Ok(PreparedHandler {
            index: observed.index,
        })
    }

    pub(super) fn attach_handler(&mut self, handler: PreparedHandler) -> Result<(), Kind> {
        let pending = self
            .facts
            .pending_handlers
            .get(handler.index)
            .ok_or(Kind::InvalidEvent)?;
        let span = pending.span;
        if pending.scope != self.scope {
            return Err(self.handler_error(span, FileIssueKind::InvalidSource));
        }
        self.with_walk(span, |file| {
            let resolution = file
                .facts
                .pending_handlers
                .get(handler.index)
                .ok_or(Kind::InvalidEvent)?
                .resolution
                .as_ref()
                .ok_or(Kind::InvalidEvent)?;
            // The owner stays parked through the checked mint. Only its normal
            // success moves that original body/facts into this exact node row.
            let (node, original) = file
                .region
                .native_on(resolution, span)
                .map_err(Kind::Artifact)?;
            let resolution = file
                .facts
                .pending_handlers
                .get_mut(handler.index)
                .ok_or(Kind::InvalidEvent)?
                .resolution
                .take()
                .ok_or(Kind::InvalidEvent)?;
            file.facts.handlers.insert(
                node,
                HandlerRecord {
                    original,
                    scope: file.scope,
                    resolution,
                },
            );
            Ok(())
        })
    }

    fn handler_error(&mut self, span: Span, kind: FileIssueKind) -> Kind {
        let _ = self.reject(span, kind);
        Kind::Handler { span, kind }
    }
}
