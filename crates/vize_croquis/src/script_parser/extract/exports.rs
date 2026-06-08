use oxc_ast::ast::{Declaration, VariableDeclarationKind};
use oxc_span::Span;

use crate::croquis::{InvalidExport, InvalidExportKind, TypeExport, TypeExportKind};
use vize_carton::CompactString;

use super::super::ScriptParseResult;

pub fn process_type_export(result: &mut ScriptParseResult, decl: &Declaration<'_>, span: Span) {
    let typeof_refs = super::super::typeof_refs::collect_from_declaration(decl);
    match decl {
        Declaration::TSTypeAliasDeclaration(type_alias) => {
            result.record_type_export(
                TypeExport {
                    name: CompactString::new(type_alias.id.name.as_str()),
                    kind: TypeExportKind::Type,
                    start: span.start,
                    end: span.end,
                    hoisted: true,
                },
                typeof_refs,
            );
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            result.record_type_export(
                TypeExport {
                    name: CompactString::new(interface.id.name.as_str()),
                    kind: TypeExportKind::Interface,
                    start: span.start,
                    end: span.end,
                    hoisted: true,
                },
                typeof_refs,
            );
        }
        _ => {}
    }
}

/// Process invalid export in script setup
pub fn process_invalid_export(result: &mut ScriptParseResult, decl: &Declaration<'_>, span: Span) {
    let (name, kind) = match decl {
        Declaration::VariableDeclaration(var_decl) => {
            let first_name = var_decl
                .declarations
                .first()
                .and_then(|d| {
                    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &d.id {
                        Some(id.name.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("unknown");

            let kind = match var_decl.kind {
                VariableDeclarationKind::Const => InvalidExportKind::Const,
                VariableDeclarationKind::Let => InvalidExportKind::Let,
                VariableDeclarationKind::Var => InvalidExportKind::Var,
                _ => InvalidExportKind::Const,
            };

            (first_name, kind)
        }
        Declaration::FunctionDeclaration(func) => {
            let name = func
                .id
                .as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("anonymous");
            (name, InvalidExportKind::Function)
        }
        Declaration::ClassDeclaration(class) => {
            let name = class
                .id
                .as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("anonymous");
            (name, InvalidExportKind::Class)
        }
        _ => return,
    };

    result.invalid_exports.push(InvalidExport {
        name: CompactString::new(name),
        kind,
        start: span.start,
        end: span.end,
    });
}
