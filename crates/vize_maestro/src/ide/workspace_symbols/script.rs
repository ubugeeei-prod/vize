//! Top-level declarations in ordinary JS/TS modules, from their authored AST.

use oxc_ast::ast::{BindingPattern, Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use tower_lsp::lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};
use vize_l0::Allocator;

pub(super) fn collect(uri: &Url, source: &str, query: &str, symbols: &mut Vec<SymbolInformation>) {
    let Ok(source_type) = SourceType::from_path(uri.path()) else {
        return;
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let mut emit = |name: &str, span: Span, kind: SymbolKind| {
        if !name.to_lowercase().contains(query) {
            return;
        }
        #[expect(
            deprecated,
            clippy::disallowed_methods,
            reason = "SymbolInformation's deprecated/name fields are the tower-lsp wire representation"
        )]
        symbols.push(SymbolInformation {
            name: name.to_string(),
            kind,
            tags: None,
            deprecated: None,
            location: Location {
                uri: uri.clone(),
                range: Range {
                    start: position(source, span.start),
                    end: position(source, span.end),
                },
            },
            container_name: None,
        });
    };
    for statement in &parsed.program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    collect_declaration(declaration, &mut emit);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    emit(id.name.as_str(), id.span, SymbolKind::FUNCTION);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    emit(id.name.as_str(), id.span, SymbolKind::CLASS);
                }
            }
            Statement::VariableDeclaration(variable) => {
                for declarator in &variable.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                        emit(id.name.as_str(), id.span, SymbolKind::VARIABLE);
                    }
                }
            }
            Statement::TSInterfaceDeclaration(value) => {
                emit(value.id.name.as_str(), value.id.span, SymbolKind::INTERFACE)
            }
            Statement::TSTypeAliasDeclaration(value) => emit(
                value.id.name.as_str(),
                value.id.span,
                SymbolKind::TYPE_PARAMETER,
            ),
            Statement::TSEnumDeclaration(value) => {
                emit(value.id.name.as_str(), value.id.span, SymbolKind::ENUM)
            }
            _ => {}
        }
    }
}

fn collect_declaration(
    declaration: &Declaration<'_>,
    emit: &mut impl FnMut(&str, Span, SymbolKind),
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                emit(id.name.as_str(), id.span, SymbolKind::FUNCTION);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                emit(id.name.as_str(), id.span, SymbolKind::CLASS);
            }
        }
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    emit(id.name.as_str(), id.span, SymbolKind::VARIABLE);
                }
            }
        }
        Declaration::TSInterfaceDeclaration(value) => {
            emit(value.id.name.as_str(), value.id.span, SymbolKind::INTERFACE)
        }
        Declaration::TSTypeAliasDeclaration(value) => emit(
            value.id.name.as_str(),
            value.id.span,
            SymbolKind::TYPE_PARAMETER,
        ),
        Declaration::TSEnumDeclaration(value) => {
            emit(value.id.name.as_str(), value.id.span, SymbolKind::ENUM)
        }
        _ => {}
    }
}

fn position(source: &str, offset: u32) -> Position {
    let (line, character) = crate::ide::offset_to_position(source, offset as usize);
    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_exports_have_exact_utf16_identifier_ranges_and_no_nested_or_comment_symbols() {
        let uri = Url::parse("file:///workspace/module.ts").unwrap();
        let source = "/* 🦀 */ export function café() { function hidden() {} }\r\n// export const counterfeit = 1\r\nexport const value = 1;\r\n";
        let mut symbols = Vec::new();
        collect(&uri, source, "", &mut symbols);
        assert_eq!(
            serde_json::to_value(symbols).unwrap(),
            serde_json::json!([
                { "name": "café", "kind": 12, "location": { "uri": uri, "range": {
                    "start": { "line": 0, "character": 25 }, "end": { "line": 0, "character": 29 }
                } } },
                { "name": "value", "kind": 13, "location": { "uri": uri, "range": {
                    "start": { "line": 2, "character": 13 }, "end": { "line": 2, "character": 18 }
                } } }
            ])
        );
    }
}
