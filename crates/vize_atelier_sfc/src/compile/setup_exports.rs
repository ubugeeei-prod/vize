//! A setup body cannot retain module exports inside its generated function.
//! Only the compile boundary uses this validator; editor analysis is unchanged.

use crate::types::{BlockLocation, SfcDescriptor, SfcError};
use oxc_ast::ast::{Declaration, ExportNamedDeclaration, Program, Statement};
use oxc_span::Span;

pub(super) fn validate(
    program: Option<&Program<'_>>,
    content: &str,
    descriptor: &SfcDescriptor<'_>,
) -> Result<(), SfcError> {
    let Some(setup) = descriptor.script_setup.as_ref() else {
        return Ok(());
    };
    // The normal full-SFC route lends its existing once-parsed program. Only
    // the existing exceptional recovery fallback needs a temporary parse.
    let allocator = oxc_allocator::Allocator::default();
    let fallback;
    let program = match program {
        Some(program) => program,
        None => {
            fallback = oxc_parser::Parser::new(&allocator, content, oxc_span::SourceType::ts())
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
        Statement::ExportNamedDeclaration(export) if !type_only(export) => Some(export.span),
        Statement::ExportDefaultDeclaration(export) => Some(export.span),
        Statement::ExportAllDeclaration(export) if !export.export_kind.is_type() => {
            Some(export.span)
        }
        Statement::TSExportAssignment(export) => Some(export.span),
        _ => None,
    }
}

fn type_only(export: &ExportNamedDeclaration<'_>) -> bool {
    if export.export_kind.is_type() {
        return true;
    }
    match &export.declaration {
        Some(Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_)) => {
            true
        }
        Some(Declaration::VariableDeclaration(value)) => value.declare,
        Some(Declaration::FunctionDeclaration(value)) => value.declare,
        Some(Declaration::ClassDeclaration(value)) => value.declare,
        Some(Declaration::TSEnumDeclaration(value)) => value.declare,
        Some(Declaration::TSModuleDeclaration(value)) => value.declare,
        Some(_) => false,
        None => {
            !export.specifiers.is_empty()
                && export.specifiers.iter().all(|value| value.export_kind.is_type())
        }
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
