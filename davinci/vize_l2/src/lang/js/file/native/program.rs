//! Attach the actual selected Program through the existing sole declaration walk.

use super::{NativeRouteState, NativeTemplateIssue, NativeTemplateIssueKind, NativeTemplateOwner};
use crate::file::ScriptUnitId;
use crate::lang::js::file::{ProgramInput, ProgramScope};
use oxc_parser::AdmittedProgram;
use vize_l0::Span;
use vize_l1::embed::Lang;

impl<'a> NativeTemplateOwner<'a> {
    pub fn ordinary_program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
    ) -> Result<ScriptUnitId, NativeTemplateIssue> {
        self.program(admitted, false)
    }
    pub fn setup_program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
    ) -> Result<ScriptUnitId, NativeTemplateIssue> {
        self.program(admitted, true)
    }
    fn program(
        &mut self,
        admitted: AdmittedProgram<'_, 'a>,
        setup: bool,
    ) -> Result<ScriptUnitId, NativeTemplateIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, NativeRouteState::Scripts) {
            return self.refuse(span, NativeTemplateIssueKind::Interrupted);
        }
        let selection = if setup {
            self.selected.setup()
        } else {
            self.selected.ordinary()
        };
        let Some(selection) = selection else {
            return self.refuse(span, NativeTemplateIssueKind::MissingProgram);
        };
        let span = selection.block().span();
        let previous = if setup { self.setup } else { self.ordinary };
        if previous.is_some() {
            return self.refuse(span, NativeTemplateIssueKind::DuplicateProgram);
        }
        let source_type = admitted.source_type();
        if !source_type.is_module()
            || source_type.is_unambiguous()
            || source_type.is_jsx()
            || source_type.is_typescript_definition()
            || source_type.is_typescript() != (selection.lang() == Lang::Ts)
        {
            return self.refuse(span, NativeTemplateIssueKind::InvalidProfile);
        }
        let input =
            match ProgramInput::checked(admitted, selection.block(), selection.container_index()) {
                Ok(input) => input,
                Err(error) => {
                    return self.refuse(error.span, NativeTemplateIssueKind::Program(error.kind));
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
                return self.refuse(error.span, NativeTemplateIssueKind::Program(error.kind));
            }
        };
        if setup {
            self.setup = Some(unit);
        } else {
            self.ordinary = Some(unit);
        }
        Ok(unit)
    }
    pub(super) fn refuse<T>(
        &mut self,
        span: Span,
        kind: NativeTemplateIssueKind,
    ) -> Result<T, NativeTemplateIssue> {
        let issue = NativeTemplateIssue { span, kind };
        self.state = NativeRouteState::Refused(issue);
        Err(issue)
    }
}
