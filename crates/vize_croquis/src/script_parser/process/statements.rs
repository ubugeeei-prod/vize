//! Top-level statement and declaration processing for Vue scripts.
//!
//! Handles processing of:
//! - Variable declarations (const, let, var)
//! - Function and class declarations
//! - Import and export statements
//! - Type declarations

use oxc_ast::ast::{Class, Declaration, Expression, Function, Statement, VariableDeclaration};
use oxc_span::GetSpan;

use crate::ScopeBinding;
use crate::croquis::{
    ImportStatementInfo, InvalidExport, InvalidExportKind, ReExportInfo, TypeExport, TypeExportKind,
};
use crate::scope::{BlockKind, BlockScopeData, ClosureScopeData, ExternalModuleScopeData};
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::ScriptParseResult;
use super::super::extract::{
    detect_setup_context_violation, process_call_expression, process_invalid_export,
    process_type_export,
};
use super::super::walk::{extract_function_params, walk_expression, walk_statement};
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

        // Module declarations (imports, exports)
        Statement::ImportDeclaration(import) => {
            result.import_statements.push(ImportStatementInfo {
                start: import.span.start,
                end: import.span.end,
            });

            let is_type_only = import.import_kind.is_type();

            // Create external module scope for this import
            let source_name = import.source.value.as_str();
            let span = import.span;

            result.scopes.enter_external_module_scope(
                ExternalModuleScopeData {
                    source: CompactString::new(source_name),
                    is_type_only,
                },
                span.start,
                span.end,
            );

            if let Some(specifiers) = &import.specifiers {
                for spec in specifiers.iter() {
                    let (name, is_type_spec, local_span) = match spec {
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                            (s.local.name.as_str(), s.import_kind.is_type(), s.local.span)
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                            (s.local.name.as_str(), false, s.local.span)
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                            (s.local.name.as_str(), false, s.local.span)
                        }
                    };

                    if source_name == "vue"
                        && let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) = spec
                    {
                        let imported = s.imported.name().as_str();
                        if is_vue_runtime_api(imported) && imported != name {
                            result
                                .reactivity_aliases
                                .insert(CompactString::new(name), CompactString::new(imported));
                            match imported {
                                "inject" => {
                                    result.inject_aliases.insert(CompactString::new(name));
                                }
                                "provide" => {
                                    result.provide_aliases.insert(CompactString::new(name));
                                }
                                _ => {}
                            }
                        }
                    }

                    // Record definition span for Go-to-Definition
                    result
                        .binding_spans
                        .insert(CompactString::new(name), (local_span.start, local_span.end));

                    // Determine binding type based on specifier kind:
                    // - Named imports (ImportSpecifier) -> SetupMaybeRef (could be ref/reactive)
                    // - Default/Namespace imports -> SetupConst
                    let binding_type = if is_type_only || is_type_spec {
                        BindingType::ExternalModule
                    } else {
                        match spec {
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(_) => {
                                BindingType::SetupMaybeRef
                            }
                            _ => BindingType::SetupConst, // default/namespace
                        }
                    };
                    result.scopes.add_binding(
                        CompactString::new(name),
                        ScopeBinding::new(binding_type, span.start),
                    );

                    // Only add to bindings if not type-only
                    if !is_type_only && !is_type_spec {
                        result.bindings.add(name, binding_type);
                        result
                            .import_sources
                            .insert(CompactString::new(name), CompactString::new(source_name));
                    }
                }
            }

            result.scopes.exit_scope();
        }

        Statement::ExportNamedDeclaration(export) => {
            // Re-export: `export { ... } from "..."`
            if export.source.is_some() {
                result.re_exports.push(ReExportInfo {
                    start: export.span.start,
                    end: export.span.end,
                });
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
                        process_type_export(result, decl, stmt.span());
                    }
                    _ => {
                        // Check if it's a type-only export (export type { ... })
                        if export.export_kind.is_type() {
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
    match decl {
        Declaration::VariableDeclaration(variable) => {
            process_variable_declaration(result, variable, source)
        }
        Declaration::FunctionDeclaration(func) => {
            process_function_declaration(result, func, source)
        }
        Declaration::ClassDeclaration(class) => process_class_declaration(result, class),
        _ => {}
    }
}

fn is_vue_runtime_api(name: &str) -> bool {
    matches!(
        name,
        "inject"
            | "provide"
            | "ref"
            | "shallowRef"
            | "reactive"
            | "shallowReactive"
            | "computed"
            | "readonly"
            | "shallowReadonly"
            | "toRef"
            | "toRefs"
            | "watch"
            | "watchEffect"
            | "watchPostEffect"
            | "watchSyncEffect"
            | "onMounted"
            | "onUnmounted"
            | "onBeforeMount"
            | "onBeforeUnmount"
            | "onUpdated"
            | "onBeforeUpdate"
            | "onActivated"
            | "onDeactivated"
            | "onWatcherCleanup"
    )
}
