use std::path::Path;
use vize_l0::ts_emit::generated_name::SHIPPED_DEFAULT;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString, cstr};

pub(super) fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_declaration_file_name)
}

pub(super) fn is_declaration_file_name(name: &str) -> bool {
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

#[derive(Debug)]
struct TextEdit {
    start: usize,
    end: usize,
    replacement: String,
}

impl TextEdit {
    fn insertion(offset: u32, text: &str) -> Self {
        Self {
            start: offset as usize,
            end: offset as usize,
            replacement: text.into(),
        }
    }
}

fn ambient_insertion(declaration: &Declaration<'_>) -> Option<u32> {
    match declaration {
        Declaration::VariableDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::FunctionDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::ClassDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::TSEnumDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        Declaration::TSModuleDeclaration(declaration) if !declaration.declare => {
            Some(declaration.span.start)
        }
        _ => None,
    }
}

fn default_export_name(text: &str, sequence: &mut usize) -> String {
    loop {
        let name = cstr!("{SHIPPED_DEFAULT}{}", *sequence);
        *sequence += 1;
        if !text.contains(name.as_str()) {
            return name;
        }
    }
}

fn keyword_end(text: &str, span_start: u32, span_end: u32, keyword: &str) -> Option<usize> {
    let start = span_start as usize;
    let end = span_end as usize;
    let source = text.get(start..end)?;
    let bytes = source.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = source.get(offset..)?;
        if remaining.starts_with("//") {
            offset += remaining.find(['\n', '\r']).unwrap_or(remaining.len());
            continue;
        }
        if remaining.starts_with("/*") {
            let close = remaining.get(2..)?.find("*/")?;
            offset += close + 4;
            continue;
        }
        if remaining.starts_with(keyword)
            && offset
                .checked_sub(1)
                .and_then(|previous| bytes.get(previous))
                .is_none_or(|previous| !is_identifier_byte(*previous))
            && bytes
                .get(offset + keyword.len())
                .is_none_or(|next| !is_identifier_byte(*next))
        {
            return Some(start + offset + keyword.len());
        }
        offset += remaining.chars().next()?.len_utf8();
    }
    None
}

const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
}

struct DefaultExport<'a> {
    export_start: u32,
    declaration_start: u32,
    declaration_end: u32,
    declared_name: Option<&'a str>,
    anonymous_name_offset: Option<u32>,
}

fn ambient_default_export(
    text: &str,
    declaration: DefaultExport<'_>,
    sequence: &mut usize,
    exported_names: &mut Vec<String>,
    edits: &mut Vec<TextEdit>,
) {
    let export_start = declaration.export_start as usize;
    let declaration_start = declaration.declaration_start as usize;
    if text.get(export_start..declaration_start).is_none() {
        return;
    }
    // OXC already identified this as an export-default declaration. Remove the
    // complete token prefix instead of searching its source spelling: comments
    // between `export`, `default`, and the declaration may contain either word.
    edits.push(TextEdit {
        start: export_start,
        end: declaration_start,
        replacement: String::default(),
    });
    edits.push(TextEdit::insertion(declaration_start as u32, "declare "));

    let name = declaration.declared_name.map_or_else(
        || default_export_name(text, sequence),
        ToCompactString::to_compact_string,
    );
    if declaration.declared_name.is_none()
        && let Some(offset) = declaration.anonymous_name_offset
    {
        edits.push(TextEdit::insertion(offset, cstr!(" {name}").as_str()));
    }
    if !exported_names.contains(&name) {
        exported_names.push(name.clone());
        edits.push(TextEdit::insertion(
            declaration.declaration_end,
            cstr!("\nexport default {name};").as_str(),
        ));
    }
}

/// Make the value declarations that are implicitly ambient in a `.d.ts`
/// explicitly ambient before an ordinary `.ts` host parses them.
pub fn project_declaration_as_typescript(text: &str) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::d_ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return text.to_compact_string();
    }

    let mut edits = Vec::new();
    let mut default_sequence = 0;
    let mut exported_default_names = Vec::new();
    for statement in &parsed.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::FunctionDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::ClassDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::TSEnumDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::TSModuleDeclaration(declaration) if !declaration.declare => {
                edits.push(TextEdit::insertion(declaration.span.start, "declare "));
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(offset) = export.declaration.as_ref().and_then(ambient_insertion) {
                    edits.push(TextEdit::insertion(offset, "declare "));
                }
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(declaration) => {
                    let anonymous_name_offset = declaration
                        .type_parameters
                        .as_ref()
                        .map_or(declaration.params.span.start, |parameters| {
                            parameters.span.start
                        });
                    ambient_default_export(
                        text,
                        DefaultExport {
                            export_start: export.span.start,
                            declaration_start: declaration.span.start,
                            declaration_end: declaration.span.end,
                            declared_name: declaration.id.as_ref().map(|id| id.name.as_str()),
                            anonymous_name_offset: Some(anonymous_name_offset),
                        },
                        &mut default_sequence,
                        &mut exported_default_names,
                        &mut edits,
                    );
                }
                ExportDefaultDeclarationKind::ClassDeclaration(declaration) => {
                    let anonymous_name_offset = declaration
                        .type_parameters
                        .as_ref()
                        .map(|parameters| parameters.span.start)
                        .or_else(|| {
                            keyword_end(text, declaration.span.start, declaration.span.end, "class")
                                .map(|offset| offset as u32)
                        });
                    ambient_default_export(
                        text,
                        DefaultExport {
                            export_start: export.span.start,
                            declaration_start: declaration.span.start,
                            declaration_end: declaration.span.end,
                            declared_name: declaration.id.as_ref().map(|id| id.name.as_str()),
                            anonymous_name_offset,
                        },
                        &mut default_sequence,
                        &mut exported_default_names,
                        &mut edits,
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }

    edits.sort_unstable_by(|left, right| {
        right
            .start
            .cmp(&left.start)
            .then_with(|| right.end.cmp(&left.end))
    });
    let mut projected = text.to_compact_string();
    for edit in edits {
        projected.replace_range(edit.start..edit.end, &edit.replacement);
    }
    projected
}
