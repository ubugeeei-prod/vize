//! A setup body cannot retain module exports inside its generated function.
//! Only the compile boundary uses this validator; editor analysis is unchanged.

use crate::compile_script::statement_sections::is_type_only_export;
use crate::types::{BlockLocation, SfcDescriptor, SfcError};
use oxc_ast::ast::{Program, Statement};
use oxc_span::Span;

pub(super) fn validate(
    program: Option<&Program<'_>>,
    content: &str,
    descriptor: &SfcDescriptor<'_>,
) -> Result<(), SfcError> {
    let Some(setup) = descriptor.script_setup.as_ref() else {
        return Ok(());
    };
    // The existing program indexes prepared text. Reuse it only when its
    // offsets still identify the authored setup body; macro erasure and lazy
    // hydration can otherwise move a later export in the original .vue file.
    let allocator = oxc_allocator::Allocator::default();
    let fallback;
    let program = match program {
        Some(program) if content == setup.content.as_ref() => program,
        _ => {
            fallback = oxc_parser::Parser::new(
                &allocator,
                setup.content.as_ref(),
                oxc_span::SourceType::ts(),
            )
            .parse();
            &fallback.program
        }
    };
    let Some(span) = program.body.iter().find_map(runtime_export) else {
        return Ok(());
    };
    let start = setup.loc.start + span.start as usize;
    let end = setup.loc.start + span.end as usize;
    Err(SfcError {
        message: "<script setup> cannot contain ES module exports.".into(),
        code: Some("SCRIPT_SETUP_EXPORT".into()),
        loc: Some(location(&descriptor.source, start, end)),
    })
}

fn runtime_export(statement: &Statement<'_>) -> Option<Span> {
    match statement {
        Statement::ExportNamedDeclaration(export) if !is_type_only_export(export) => {
            Some(export.span)
        }
        Statement::ExportDefaultDeclaration(export) => Some(export.span),
        Statement::ExportAllDeclaration(export) if !export.export_kind.is_type() => {
            Some(export.span)
        }
        Statement::TSExportAssignment(export) => Some(export.span),
        _ => None,
    }
}

fn location(source: &str, start: usize, end: usize) -> BlockLocation {
    let position = |offset| {
        let mut line = 1;
        let mut column = 1;
        for (index, character) in source.char_indices() {
            if index >= offset {
                break;
            }
            if character == '\n' {
                line += 1;
                column = 1;
            } else {
                column += character.len_utf16();
            }
        }
        (line, column)
    };
    let (start_line, start_column) = position(start);
    let (end_line, end_column) = position(end);
    BlockLocation {
        start,
        end,
        tag_start: start,
        tag_end: end,
        start_line,
        start_column,
        end_line,
        end_column,
    }
}
