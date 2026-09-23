//! Top-level statement and declaration processing for Vue scripts.
//!
//! Handles processing of:
//! - Variable declarations (const, let, var)
//! - Function and class declarations
//! - Import and export statements
//! - Type declarations

use oxc_ast::ast::{Class, Declaration, Expression, Function, Statement, VariableDeclaration};
use oxc_span::GetSpan;

use crate::croquis::{
    InvalidExport, InvalidExportKind, ReExportForward, ReExportInfo, TypeExport, TypeExportKind,
};
use crate::scope::{BlockKind, BlockScopeData, ClosureScopeData};
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::ScriptParseResult;
use super::super::extract::{
    detect_setup_context_violation, process_call_expression, process_invalid_export,
    process_type_export,
};
use super::super::walk::{extract_function_params, walk_expression, walk_statement};
use super::enums::process_enum_declaration;
use super::macros;

/// Process a single statement
pub fn process_statement(result: &mut ScriptParseResult, stmt: &Statement<'_>, source: &str) {
    match stmt {
        // Variable declarations: const, let, var
        Statement::VariableDeclaration(decl) => process_variable_declaration(result, decl, source),

        // Function declarations
        Statement::FunctionDeclaration(func) => process_function_declaration(result, func, source),

        // Class declarations
        Statement::ClassDeclaration(class) => process_class_declaration(result, class),

        // Runtime TypeScript enums are setup values; erased enums stay type-only.
        Statement::TSEnumDeclaration(enumeration) => process_enum_declaration(result, enumeration),

        // Expression statements (may contain macro calls and callback scopes)
        Statement::ExpressionStatement(expr_stmt) => {
            if let Expression::CallExpression(call) = &expr_stmt.expression {
                // Detect setup context violations (watch, onMounted, etc.)
                detect_setup_context_violation(result, call);
                process_call_expression(result, call, source);
            }
            // Walk the expression to find callback scopes
            walk_expression(result, &expr_stmt.expression, source);
        }

        Statement::ImportDeclaration(import) => super::imports::process_import(result, import),

        Statement::ExportNamedDeclaration(export) => {
            // Re-export: `export { ... } from "..."`
            if export.source.is_some() {
                result.re_exports.push(ReExportInfo {
                    start: export.span.start,
                    end: export.span.end,
                });
                record_reexport_forwards(result, export);
                return;
            }

            // Specifier-only export without a declaration, e.g.
            // `export { Foo }` / `export type { Foo }`. These re-export
            // local or imported bindings and are only valid at module top
            // level, so lift them out of the synthetic `__setup` function.
            if export.declaration.is_none() {
                result.re_exports.push(ReExportInfo {
                    start: export.span.start,
                    end: export.span.end,
                });
                return;
            }

            if let Some(decl) = &export.declaration {
                // Check if the declaration itself is a type declaration
                match decl {
                    Declaration::TSTypeAliasDeclaration(_)
                    | Declaration::TSInterfaceDeclaration(_) => {
                        // Type exports are valid in script setup
                        result.register_local_type(decl, source);
                        process_type_export(result, decl, stmt.span());
                    }
                    _ => {
                        // Check if it's a type-only export (export type { ... })
                        if export.export_kind.is_type() {
                            result.register_local_type(decl, source);
                            process_type_export(result, decl, stmt.span());
                        } else if result.is_non_setup_script {
                            // Plain <script> exports stay in the synthetic setup
                            // scope, so keep their bindings available to the template.
                            process_exported_value_declaration(result, decl, source);
                        } else if !result.is_non_setup_script {
                            // Value exports are invalid in script setup
                            process_invalid_export(result, decl, stmt.span());
                        }
                    }
                }
            }
        }

        Statement::ExportDefaultDeclaration(export) if !result.is_non_setup_script => {
            // Default exports are invalid in script setup
            result.invalid_exports.push(InvalidExport {
                name: CompactString::new("default"),
                kind: InvalidExportKind::Default,
                start: export.span.start,
                end: export.span.end,
            });
        }

        // Type declarations at top level
        Statement::TSTypeAliasDeclaration(type_alias) => {
            // Type aliases are allowed (not bindings, but tracked)
            let name = type_alias.id.name.as_str();
            result.register_local_type_alias(type_alias, source);
            let typeof_refs = super::super::typeof_refs::collect_from_type_alias(type_alias);
            result.record_type_export(
                TypeExport {
                    name: CompactString::new(name),
                    kind: TypeExportKind::Type,
                    start: type_alias.span.start,
                    end: type_alias.span.end,
                    hoisted: true,
                },
                typeof_refs,
            );
        }

        Statement::TSInterfaceDeclaration(interface) => {
            // Interfaces are allowed (not bindings, but tracked)
            let name = interface.id.name.as_str();
            result.register_local_interface(interface, source);
            let typeof_refs = super::super::typeof_refs::collect_from_interface(interface);
            result.record_type_export(
                TypeExport {
                    name: CompactString::new(name),
                    kind: TypeExportKind::Interface,
                    start: interface.span.start,
                    end: interface.span.end,
                    hoisted: true,
                },
                typeof_refs,
            );
        }

        // Block statements at top level (scoped blocks)
        Statement::BlockStatement(block) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::Block,
                },
                block.span.start,
                block.span.end,
            );
            for stmt in block.body.iter() {
                walk_statement(result, stmt, source);
            }
            result.scopes.exit_scope();
        }

        _ => {}
    }
}

fn process_variable_declaration(
    result: &mut ScriptParseResult,
    decl: &VariableDeclaration<'_>,
    source: &str,
) {
    for declarator in decl.declarations.iter() {
        macros::process_variable_declarator(result, declarator, decl.kind, source);
    }
}

fn process_function_declaration(result: &mut ScriptParseResult, func: &Function<'_>, source: &str) {
    if let Some(id) = &func.id {
        let name = id.name.as_str();
        result.bindings.add(name, BindingType::SetupConst);
        result
            .binding_spans
            .insert(CompactString::new(name), (id.span.start, id.span.end));
    }

    // Create closure scope and walk body
    let params = extract_function_params(&func.params);
    let name = func
        .id
        .as_ref()
        .map(|id| CompactString::new(id.name.as_str()));

    result.scopes.enter_closure_scope(
        ClosureScopeData {
            name,
            param_names: params,
            is_arrow: false,
            is_async: func.r#async,
            is_generator: func.generator,
        },
        func.span.start,
        func.span.end,
    );

    if let Some(body) = &func.body {
        for stmt in body.statements.iter() {
            walk_statement(result, stmt, source);
        }
    }

    result.scopes.exit_scope();
}

fn process_class_declaration(result: &mut ScriptParseResult, class: &Class<'_>) {
    if let Some(id) = &class.id {
        let name = id.name.as_str();
        result.bindings.add(name, BindingType::SetupConst);
        result
            .binding_spans
            .insert(CompactString::new(name), (id.span.start, id.span.end));
    }
}
fn process_exported_value_declaration(
    result: &mut ScriptParseResult,
    decl: &Declaration<'_>,
    source: &str,
) {
    super::module_exports::record_module_value_exports(result, decl);
    match decl {
        Declaration::VariableDeclaration(variable) => {
            process_variable_declaration(result, variable, source)
        }
        Declaration::FunctionDeclaration(func) => {
            process_function_declaration(result, func, source)
        }
        Declaration::ClassDeclaration(class) => process_class_declaration(result, class),
        Declaration::TSEnumDeclaration(enumeration) => {
            process_enum_declaration(result, enumeration)
        }
        _ => {}
    }
}

fn record_reexport_forwards(
    result: &mut ScriptParseResult,
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
) {
    if export.export_kind.is_type() {
        return;
    }
    let Some(source) = export.source.as_ref() else {
        return;
    };
    for specifier in &export.specifiers {
        if specifier.export_kind.is_type() {
            continue;
        }
        result.re_export_forwards.push(ReExportForward {
            source: CompactString::new(source.value.as_str()),
            imported: CompactString::new(specifier.local.name().as_str()),
            exported: CompactString::new(specifier.exported.name().as_str()),
        });
    }
}
