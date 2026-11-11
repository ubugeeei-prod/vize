//! Declaration exports count as uses. Specifier exports are ordinary references.

use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, Declaration, ExportDefaultDeclarationKind, Statement,
    VariableDeclaration,
};
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashMap;

use super::classify::BindingInfo;

pub(super) fn mark_exports(
    program: &oxc_ast::ast::Program<'_>,
    infos: &mut FxHashMap<SymbolId, BindingInfo>,
) {
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    mark_declaration(declaration, infos);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                mark_default(&export.declaration, infos);
            }
            _ => {}
        }
    }
}

fn mark_declaration(declaration: &Declaration<'_>, infos: &mut FxHashMap<SymbolId, BindingInfo>) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => mark_variables(declaration, infos),
        Declaration::FunctionDeclaration(function) => mark_id(function.id.as_ref(), infos),
        Declaration::ClassDeclaration(class) => mark_id(class.id.as_ref(), infos),
        Declaration::TSTypeAliasDeclaration(declaration) => mark_id(Some(&declaration.id), infos),
        Declaration::TSInterfaceDeclaration(declaration) => mark_id(Some(&declaration.id), infos),
        Declaration::TSEnumDeclaration(declaration) => mark_id(Some(&declaration.id), infos),
        Declaration::TSImportEqualsDeclaration(declaration) => {
            mark_id(Some(&declaration.id), infos)
        }
        Declaration::TSModuleDeclaration(_) | Declaration::TSGlobalDeclaration(_) => {}
    }
}

fn mark_variables(
    declaration: &VariableDeclaration<'_>,
    infos: &mut FxHashMap<SymbolId, BindingInfo>,
) {
    for declarator in &declaration.declarations {
        mark_pattern(&declarator.id, infos);
    }
}

fn mark_default(
    declaration: &ExportDefaultDeclarationKind<'_>,
    infos: &mut FxHashMap<SymbolId, BindingInfo>,
) {
    match declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            mark_id(function.id.as_ref(), infos);
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => mark_id(class.id.as_ref(), infos),
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(declaration) => {
            mark_id(Some(&declaration.id), infos);
        }
        _ => {}
    }
}

fn mark_pattern(pattern: &BindingPattern<'_>, infos: &mut FxHashMap<SymbolId, BindingInfo>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => mark_id(Some(id), infos),
        BindingPattern::AssignmentPattern(assignment) => mark_pattern(&assignment.left, infos),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                mark_pattern(&property.value, infos);
            }
            if let Some(rest) = &object.rest {
                mark_pattern(&rest.argument, infos);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                mark_pattern(element, infos);
            }
            if let Some(rest) = &array.rest {
                mark_pattern(&rest.argument, infos);
            }
        }
    }
}

fn mark_id(id: Option<&BindingIdentifier<'_>>, infos: &mut FxHashMap<SymbolId, BindingInfo>) {
    let Some(id) = id else {
        return;
    };
    let Some(symbol_id) = id.symbol_id.get() else {
        return;
    };
    if let Some(info) = infos.get_mut(&symbol_id) {
        info.exported = true;
    }
}
