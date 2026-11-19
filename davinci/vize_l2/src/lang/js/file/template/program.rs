//! Attach the actual selected Program through the existing sole declaration walk.

use super::{TemplateRouteState, TemplateRouteIssue, TemplateRouteIssueKind, TemplateOwner};
use crate::file::ScriptUnitId;
use crate::lang::js::file::{FileProducer, ProgramInput, ProgramScope};
use oxc_parser::AdmittedProgram;
use vize_l0::Span;
use vize_l1::embed::Lang;
use vize_l1::markup::NativeTemplateComponent;

impl<'a> TemplateOwner<'a> {
    pub fn ordinary_program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
    ) -> Result<ScriptUnitId, TemplateRouteIssue> {
        self.program(admitted, false)
    }
    pub fn setup_program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
    ) -> Result<ScriptUnitId, TemplateRouteIssue> {
        self.program(admitted, true)
    }
    fn program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
        setup: bool,
    ) -> Result<ScriptUnitId, TemplateRouteIssue> {
        Parts {
            selected: &self.selected,
            producer: &mut self.producer,
            state: &mut self.state,
            ordinary: &mut self.ordinary,
            setup: &mut self.setup,
        }
        .attach(admitted, setup)
    }
    pub(super) fn refuse<T>(
        &mut self,
        span: Span,
        kind: TemplateRouteIssueKind,
    ) -> Result<T, TemplateRouteIssue> {
        let issue = TemplateRouteIssue { span, kind };
        self.state = TemplateRouteState::Refused(issue);
        Err(issue)
    }
}

/// Private field borrows preserve the original owner while its syntax is borrowed.
pub(super) struct Parts<'owner, 'arena> {
    pub(super) selected: &'owner NativeTemplateComponent<'arena>,
    pub(super) producer: &'owner mut FileProducer<'arena>,
    pub(super) state: &'owner mut TemplateRouteState,
    pub(super) ordinary: &'owner mut Option<ScriptUnitId>,
    pub(super) setup: &'owner mut Option<ScriptUnitId>,
}
impl<'arena> Parts<'_, 'arena> {
    pub(super) fn attach(
        &mut self,
        admitted: AdmittedProgram<'_, 'arena>,
        setup: bool,
    ) -> Result<ScriptUnitId, TemplateRouteIssue> {
        let span = self.selected.component().block().span();
        if !matches!(*self.state, TemplateRouteState::Scripts) {
            return self.refuse(span, TemplateRouteIssueKind::Interrupted);
        }
        *self.state = TemplateRouteState::Interrupted;
        let selection = if setup {
            self.selected.setup()
        } else {
            self.selected.ordinary()
        };
        let Some(selection) = selection else {
            return self.refuse(span, TemplateRouteIssueKind::MissingProgram);
        };
        let span = selection.block().span();
        let previous = if setup { *self.setup } else { *self.ordinary };
        if previous.is_some() {
            return self.refuse(span, TemplateRouteIssueKind::DuplicateProgram);
        }
        let source_type = admitted.source_type();
        if !source_type.is_module()
            || source_type.is_unambiguous()
            || source_type.is_jsx()
            || source_type.is_typescript_definition()
            || source_type.is_typescript() != (selection.lang() == Lang::Ts)
        {
            return self.refuse(span, TemplateRouteIssueKind::InvalidProfile);
        }
        let input =
            match ProgramInput::checked(admitted, selection.block(), selection.container_index()) {
                Ok(input) => input,
                Err(error) => {
                    return self.refuse(error.span, TemplateRouteIssueKind::Program(error.kind));
                }
            };
        // Delegate the same actual Program entry and its single declaration walk.
        let result = self.producer.program(
            input,
            if setup {
                ProgramScope::Nested
            } else {
                ProgramScope::Module
            },
        );
        let unit = match result {
            Ok(unit) => unit,
            Err(error) => {
                return self.refuse(error.span, TemplateRouteIssueKind::Program(error.kind));
            }
        };
        if setup {
            *self.setup = Some(unit);
        } else {
            *self.ordinary = Some(unit);
        }
        *self.state = TemplateRouteState::Scripts;
        Ok(unit)
    }

    fn refuse<T>(
        &mut self,
        span: Span,
        kind: TemplateRouteIssueKind,
    ) -> Result<T, TemplateRouteIssue> {
        let issue = TemplateRouteIssue { span, kind };
        *self.state = TemplateRouteState::Refused(issue);
        Err(issue)
    }
}
