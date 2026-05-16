//! Runtime binding collection for script-level macro disambiguation.

use oxc_ast::ast::{
    BindingPattern, Declaration, ImportDeclarationSpecifier, Statement, VariableDeclaration,
};
use vize_carton::{FxHashSet, String};

pub(super) fn collect_runtime_bindings<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
) -> FxHashSet<String> {
    let mut bindings = FxHashSet::default();

    for stmt in statements {
        collect_runtime_bindings_from_statement(stmt, &mut bindings);
    }

    bindings
}

fn collect_runtime_bindings_from_statement(stmt: &Statement<'_>, bindings: &mut FxHashSet<String>) {
    match stmt {
        Statement::ImportDeclaration(import_decl) => {
            if import_decl.import_kind.is_type() {
                return;
            }
            if let Some(specifiers) = import_decl.specifiers.as_ref() {
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                            if !spec.import_kind.is_type() {
                                bindings.insert(spec.local.name.as_str().into());
                            }
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                            bindings.insert(spec.local.name.as_str().into());
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                            bindings.insert(spec.local.name.as_str().into());
                        }
                    }
                }
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            collect_variable_declaration_bindings(var_decl, bindings);
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(id) = func.id.as_ref() {
                bindings.insert(id.name.as_str().into());
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = class.id.as_ref() {
                bindings.insert(id.name.as_str().into());
            }
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            if export_decl.export_kind.is_type() {
                return;
            }
            if let Some(declaration) = export_decl.declaration.as_ref() {
                collect_runtime_bindings_from_declaration(declaration, bindings);
            }
        }
        _ => {}
    }
}

fn collect_runtime_bindings_from_declaration(
    declaration: &Declaration<'_>,
    bindings: &mut FxHashSet<String>,
) {
    match declaration {
        Declaration::VariableDeclaration(var_decl) => {
            collect_variable_declaration_bindings(var_decl, bindings);
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = func.id.as_ref() {
                bindings.insert(id.name.as_str().into());
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = class.id.as_ref() {
                bindings.insert(id.name.as_str().into());
            }
        }
        _ => {}
    }
}

fn collect_variable_declaration_bindings(
    var_decl: &VariableDeclaration<'_>,
    bindings: &mut FxHashSet<String>,
) {
    for declarator in var_decl.declarations.iter() {
        collect_binding_pattern_names(&declarator.id, bindings);
    }
}

fn collect_binding_pattern_names(pattern: &BindingPattern<'_>, bindings: &mut FxHashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            bindings.insert(id.name.as_str().into());
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                collect_binding_pattern_names(&prop.value, bindings);
            }
            if let Some(rest) = obj.rest.as_ref() {
                collect_binding_pattern_names(&rest.argument, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                collect_binding_pattern_names(element, bindings);
            }
            if let Some(rest) = arr.rest.as_ref() {
                collect_binding_pattern_names(&rest.argument, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_pattern_names(&assign.left, bindings);
        }
    }
}
