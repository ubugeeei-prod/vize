//! Statement and variable processing for Vue scripts.
//!
//! Handles processing of:
//! - Variable declarations (const, let, var)
//! - Function and class declarations
//! - Import and export statements
//! - Type declarations
//!
//! This module is split into:
//! - `macros`: Variable declarator processing (macros, reactivity, inject)
//! - `bindings`: Binding pattern helpers and expression classification

mod bindings;
mod macros;

use oxc_ast::ast::{
    Argument, CallExpression, Declaration, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_span::GetSpan;

use crate::analysis::{
    ComponentRegistration, ImportStatementInfo, InvalidExport, InvalidExportKind, ReExportInfo,
    TypeExport, TypeExportKind,
};
use crate::scope::{BlockKind, BlockScopeData, ClosureScopeData, ExternalModuleScopeData};
use crate::ScopeBinding;
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::extract::{
    detect_setup_context_violation, process_call_expression, process_invalid_export,
    process_type_export,
};
use super::walk::{extract_function_params, walk_expression, walk_statement};
use super::ScriptParseResult;

/// Process a single statement
pub fn process_statement(result: &mut ScriptParseResult, stmt: &Statement<'_>, source: &str) {
    match stmt {
        // Variable declarations: const, let, var
        Statement::VariableDeclaration(decl) => {
            for declarator in decl.declarations.iter() {
                macros::process_variable_declarator(result, declarator, decl.kind, source);
            }
        }

        // Function declarations
        Statement::FunctionDeclaration(func) => {
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

        // Class declarations
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                let name = id.name.as_str();
                result.bindings.add(name, BindingType::SetupConst);
                result
                    .binding_spans
                    .insert(CompactString::new(name), (id.span.start, id.span.end));
            }
        }

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

                    if source_name == "vue" {
                        if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) = spec {
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
                        } else {
                            // Value exports are invalid in script setup
                            process_invalid_export(result, decl, stmt.span());
                        }
                    }
                }
            }
        }

        Statement::ExportDefaultDeclaration(export) => {
            if result.is_non_setup_script {
                collect_options_api_component_registrations(result, &export.declaration);
            }

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
            result.type_exports.push(TypeExport {
                name: CompactString::new(name),
                kind: TypeExportKind::Type,
                start: type_alias.span.start,
                end: type_alias.span.end,
                hoisted: true,
            });
        }

        Statement::TSInterfaceDeclaration(interface) => {
            // Interfaces are allowed (not bindings, but tracked)
            let name = interface.id.name.as_str();
            result.type_exports.push(TypeExport {
                name: CompactString::new(name),
                kind: TypeExportKind::Interface,
                start: interface.span.start,
                end: interface.span.end,
                hoisted: true,
            });
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

fn collect_options_api_component_registrations(
    result: &mut ScriptParseResult,
    declaration: &ExportDefaultDeclarationKind<'_>,
) {
    let Some(options) = component_options_from_export(declaration) else {
        return;
    };

    let Some(components) = option_object_property(options, "components") else {
        return;
    };

    for property in &components.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }

        let Some(name) = property_key_name(&property.key) else {
            continue;
        };

        let local_name = if property.shorthand {
            name
        } else {
            let Expression::Identifier(identifier) = &property.value else {
                continue;
            };
            identifier.name.as_str()
        };

        result.component_registrations.push(ComponentRegistration {
            name: CompactString::new(name),
            local_name: CompactString::new(local_name),
        });
    }
}

fn component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => component_options_from_call(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }

    let first_arg = call.arguments.first()?;
    component_options_from_argument(first_arg)
}

fn component_options_from_argument<'a>(
    argument: &'a Argument<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::CallExpression(call) => component_options_from_call(call),
        Argument::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Argument::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        match &property.value {
            Expression::ObjectExpression(object) => Some(object.as_ref()),
            _ => None,
        }
    })
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
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
