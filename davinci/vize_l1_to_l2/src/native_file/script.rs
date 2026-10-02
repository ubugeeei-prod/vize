use super::{NativeScriptObservation, NativeSfcIssue, NativeSfcIssueKind};
use crate::vue_file::VueFileProducer;
use alloc::vec::Vec;
use vize_l0::Allocator;
use vize_l1::container::vue::{ScriptRole, ScriptView};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::ProgramInput;

pub(super) fn observe<'a>(
    allocator: &'a Allocator,
    selected: ScriptView<'_, 'a>,
    producer: Option<&mut VueFileProducer<'a>>,
    issues: &mut Vec<NativeSfcIssue>,
) -> NativeScriptObservation<'a> {
    let block = selected.block();
    let reject = |issues: &mut Vec<NativeSfcIssue>, kind| {
        issues.push(NativeSfcIssue {
            container_index: Some(selected.container_index()),
            span: Some(block.span()),
            kind,
        });
    };
    let mut observed = NativeScriptObservation {
        container_index: selected.container_index(),
        block,
        role: selected.role(),
        lang: selected.lang(),
        syntax: None,
        unit: None,
    };
    let source = match EmbedSource::authored(selected.source(), block.span()) {
        Ok(source) => source,
        Err(error) => {
            reject(issues, NativeSfcIssueKind::ScriptSource(error));
            return observed;
        }
    };
    let syntax = parse_program_once(allocator, source, ProgramOptions::module(selected.lang()));
    match syntax.admitted_program() {
        Some(admitted) => {
            let program = admitted.program();
            if program.body.is_empty()
                && program.comments.is_empty()
                && program.directives.is_empty()
                && program.hashbang.is_none()
            {
                reject(issues, NativeSfcIssueKind::EmptyScriptSelection);
            } else if let Some(producer) = producer {
                match ProgramInput::checked(admitted, block, selected.container_index()) {
                    Ok(input) => {
                        let result = match selected.role() {
                            ScriptRole::Ordinary => producer.ordinary(input),
                            ScriptRole::Setup => producer.setup(input),
                        };
                        match result {
                            Ok(unit) => observed.unit = Some(unit),
                            Err(error) => reject(issues, NativeSfcIssueKind::VueScript(error)),
                        }
                    }
                    Err(error) => reject(issues, NativeSfcIssueKind::ScriptInput(error)),
                }
            }
        }
        None => reject(issues, NativeSfcIssueKind::ScriptSyntax(syntax.hole())),
    }
    observed.syntax = Some(syntax);
    observed
}
