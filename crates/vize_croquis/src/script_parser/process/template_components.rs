//! Registration inventory from the existing top-level statement walk.

use super::super::ScriptParseResult;
use super::bindings::for_each_binding_pattern_name;
use crate::scope::ScopeKind;
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Statement, VariableDeclaration};
use vize_carton::CompactString;

pub(super) fn collect(result: &mut ScriptParseResult, statement: &Statement<'_>) {
    if result.skip_diagnostics
        || !matches!(
            result.scopes.current_scope().kind,
            ScopeKind::ScriptSetup | ScopeKind::NonScriptSetup
        )
    {
        return;
    }
    match statement {
        Statement::VariableDeclaration(declaration) => variables(result, declaration),
        Statement::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                name(result, id.name.as_str());
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                name(result, id.name.as_str());
            }
        }
        Statement::TSTypeAliasDeclaration(alias) => name(result, alias.id.name.as_str()),
        Statement::TSInterfaceDeclaration(interface) => name(result, interface.id.name.as_str()),
        Statement::TSEnumDeclaration(enumeration) => name(result, enumeration.id.name.as_str()),
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                declared(result, declaration);
            }
        }
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    name(result, id.name.as_str());
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    name(result, id.name.as_str());
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn declared(result: &mut ScriptParseResult, declaration: &Declaration<'_>) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => variables(result, declaration),
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                name(result, id.name.as_str());
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                name(result, id.name.as_str());
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => name(result, alias.id.name.as_str()),
        Declaration::TSInterfaceDeclaration(interface) => name(result, interface.id.name.as_str()),
        Declaration::TSEnumDeclaration(enumeration) => name(result, enumeration.id.name.as_str()),
        _ => {}
    }
}

fn variables(result: &mut ScriptParseResult, declaration: &VariableDeclaration<'_>) {
    for variable in &declaration.declarations {
        for_each_binding_pattern_name(&variable.id, &mut |binding| name(result, binding));
    }
}

pub(super) fn name(result: &mut ScriptParseResult, name: &str) {
    if result.skip_diagnostics {
        return;
    }
    result
        .template_component_registrations
        .module_names
        .insert(CompactString::new(name));
}
