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
    Argument, ArrayExpression, ArrayExpressionElement, BindingPattern, CallExpression, Class,
    Declaration, ExportDefaultDeclarationKind, Expression, Function, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement, VariableDeclaration,
};
use oxc_span::GetSpan;

use crate::ScopeBinding;
use crate::analysis::{
    ComponentRegistration, ImportStatementInfo, InvalidExport, InvalidExportKind, ReExportInfo,
    TypeExport, TypeExportKind,
};
use crate::scope::{BlockKind, BlockScopeData, ClosureScopeData, ExternalModuleScopeData};
use vize_carton::{CompactString, FxHashMap, FxHashSet, String};
use vize_relief::BindingType;

use super::ScriptParseResult;
use super::extract::{
    detect_setup_context_violation, process_call_expression, process_invalid_export,
    process_type_export,
};
use super::walk::{extract_function_params, walk_expression, walk_statement};

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
            let typeof_refs = super::typeof_refs::collect_from_type_alias(type_alias);
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
            let typeof_refs = super::typeof_refs::collect_from_interface(interface);
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

#[derive(Clone, Copy)]
struct ComponentOptionsRef<'a> {
    object: &'a ObjectExpression<'a>,
}

pub(in crate::script_parser) fn collect_options_api_component_metadata(
    result: &mut ScriptParseResult,
    program: &Program<'_>,
    legacy_vue2: bool,
) {
    let mut object_bindings = FxHashMap::default();
    collect_object_bindings(program, &mut object_bindings);

    let mut component_option_bindings = FxHashMap::default();
    collect_component_options_bindings(program, &mut component_option_bindings);

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };

        let Some(options) =
            component_options_from_export(&export.declaration, &component_option_bindings)
        else {
            continue;
        };
        collect_component_registrations_from_options(result, options.object, &object_bindings);
        if legacy_vue2 {
            collect_legacy_vue2_template_bindings_from_options(
                result,
                options.object,
                &object_bindings,
            );
        }
    }

    if legacy_vue2 {
        add_nuxt2_template_globals(result);
    }
}

fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
    object_bindings: &mut FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };

        for declarator in declaration.declarations.iter() {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let Some(object) = object_expression_from_expression(init) else {
                continue;
            };
            object_bindings.insert(id.name.as_str(), object);
        }
    }
}

fn collect_component_options_bindings<'a>(
    program: &'a Program<'a>,
    bindings: &mut FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) {
    let mut changed = true;

    while changed {
        changed = false;

        for statement in program.body.iter() {
            let Statement::VariableDeclaration(declaration) = statement else {
                continue;
            };

            for declarator in declaration.declarations.iter() {
                let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                    continue;
                };
                if bindings.contains_key(id.name.as_str()) {
                    continue;
                }
                let Some(init) = declarator.init.as_ref() else {
                    continue;
                };
                let Some(options) = component_options_from_expression(init, bindings) else {
                    continue;
                };

                bindings.insert(id.name.as_str(), options);
                changed = true;
            }
        }
    }
}

fn collect_component_registrations_from_options<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    let Some(components) = option_object_property(options, "components", object_bindings) else {
        return;
    };

    let mut seen = FxHashSet::default();
    collect_component_registrations_from_components_object(
        result,
        components,
        object_bindings,
        &mut seen,
        &mut FxHashSet::default(),
    );
}

fn collect_legacy_vue2_template_bindings_from_options<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    collect_props_bindings(result, options, object_bindings);
    collect_object_option_bindings(
        result,
        options,
        object_bindings,
        "computed",
        BindingType::Options,
    );
    collect_object_option_bindings(
        result,
        options,
        object_bindings,
        "methods",
        BindingType::Options,
    );
    collect_object_option_bindings(
        result,
        options,
        object_bindings,
        "inject",
        BindingType::Options,
    );
    collect_returned_object_option_bindings(
        result,
        options,
        object_bindings,
        "data",
        BindingType::Data,
    );
    collect_returned_object_option_bindings(
        result,
        options,
        object_bindings,
        "asyncData",
        BindingType::Data,
    );
    collect_returned_object_option_bindings(
        result,
        options,
        object_bindings,
        "setup",
        BindingType::Options,
    );
}

fn add_nuxt2_template_globals(result: &mut ScriptParseResult) {
    for name in [
        "$config",
        "$fetchState",
        "$nuxt",
        "$route",
        "$router",
        "$store",
    ] {
        add_template_binding(result, name, BindingType::VueGlobal, 0, 0);
    }
}

fn collect_props_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    let Some(props) = option_expression_property(options, "props") else {
        return;
    };

    match props {
        Expression::ArrayExpression(array) => {
            collect_array_string_bindings(result, array, BindingType::Props);
        }
        Expression::ObjectExpression(object) => {
            collect_object_property_bindings(result, object, BindingType::Props);
        }
        Expression::Identifier(identifier) => {
            if let Some(object) = object_bindings.get(identifier.name.as_str()).copied() {
                collect_object_property_bindings(result, object, BindingType::Props);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_props_bindings_from_expression(
                result,
                &parenthesized.expression,
                object_bindings,
            )
        }
        Expression::TSAsExpression(ts_as) => {
            collect_props_bindings_from_expression(result, &ts_as.expression, object_bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => collect_props_bindings_from_expression(
            result,
            &ts_satisfies.expression,
            object_bindings,
        ),
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_props_bindings_from_expression(result, &ts_non_null.expression, object_bindings)
        }
        _ => {}
    }
}

fn collect_props_bindings_from_expression<'a>(
    result: &mut ScriptParseResult,
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    match expression {
        Expression::ArrayExpression(array) => {
            collect_array_string_bindings(result, array, BindingType::Props);
        }
        Expression::ObjectExpression(object) => {
            collect_object_property_bindings(result, object, BindingType::Props);
        }
        Expression::Identifier(identifier) => {
            if let Some(object) = object_bindings.get(identifier.name.as_str()).copied() {
                collect_object_property_bindings(result, object, BindingType::Props);
            }
        }
        _ => {}
    }
}

fn collect_object_option_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    key_name: &str,
    binding_type: BindingType,
) {
    let Some(object) = option_object_property(options, key_name, object_bindings) else {
        return;
    };

    collect_object_property_bindings(result, object, binding_type);
}

fn collect_returned_object_option_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    key_name: &str,
    binding_type: BindingType,
) {
    let Some(expression) = option_expression_property(options, key_name) else {
        return;
    };
    let Some(object) = returned_object_from_expression(expression, object_bindings) else {
        return;
    };

    collect_object_property_bindings(result, object, binding_type);
}

fn collect_array_string_bindings(
    result: &mut ScriptParseResult,
    array: &ArrayExpression<'_>,
    binding_type: BindingType,
) {
    for element in &array.elements {
        let ArrayExpressionElement::StringLiteral(literal) = element else {
            continue;
        };
        let name = normalize_template_binding_name(literal.value.as_str());
        if let Some(name) = name {
            let start = literal.span.start.saturating_add(1);
            let end = literal.span.end.saturating_sub(1);
            add_template_binding(result, name.as_str(), binding_type, start, end);
        }
    }
}

fn collect_object_property_bindings(
    result: &mut ScriptParseResult,
    object: &ObjectExpression<'_>,
    binding_type: BindingType,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }

        let Some(raw_name) = property_key_name(&property.key) else {
            continue;
        };
        let Some(name) = normalize_template_binding_name(raw_name) else {
            continue;
        };
        let span = property.key.span();
        add_template_binding(result, name.as_str(), binding_type, span.start, span.end);
    }
}

fn add_template_binding(
    result: &mut ScriptParseResult,
    name: &str,
    binding_type: BindingType,
    start: u32,
    end: u32,
) {
    result.bindings.add(name, binding_type);
    result
        .binding_spans
        .entry(CompactString::new(name))
        .or_insert((start, end));
    result.scopes.add_binding(
        CompactString::new(name),
        ScopeBinding::new(binding_type, start),
    );
}

fn normalize_template_binding_name(name: &str) -> Option<CompactString> {
    if is_valid_template_binding_name(name) {
        return Some(CompactString::new(name));
    }

    if name.contains('-') {
        let camel = kebab_to_camel(name);
        if is_valid_template_binding_name(&camel) {
            return Some(CompactString::new(camel));
        }
    }

    None
}

fn is_valid_template_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn kebab_to_camel(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut upper_next = false;
    for ch in name.chars() {
        if ch == '-' {
            upper_next = true;
        } else if upper_next {
            result.push(ch.to_ascii_uppercase());
            upper_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn collect_component_registrations_from_components_object<'a>(
    result: &mut ScriptParseResult,
    components: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen: &mut FxHashSet<(CompactString, CompactString)>,
    visited_spreads: &mut FxHashSet<&'a str>,
) {
    for property in &components.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }

                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };

                let local_name = if property.shorthand {
                    name
                } else {
                    let Some(local_name) = local_name_from_expression(&property.value) else {
                        continue;
                    };
                    local_name
                };

                let pair = (CompactString::new(name), CompactString::new(local_name));
                if seen.insert(pair.clone()) {
                    result.component_registrations.push(ComponentRegistration {
                        name: pair.0,
                        local_name: pair.1,
                    });
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Expression::Identifier(identifier) = &spread.argument else {
                    continue;
                };
                let name = identifier.name.as_str();
                if !visited_spreads.insert(name) {
                    continue;
                }
                let Some(object) = object_bindings.get(name).copied() else {
                    continue;
                };
                collect_component_registrations_from_components_object(
                    result,
                    object,
                    object_bindings,
                    seen,
                    visited_spreads,
                );
            }
        }
    }
}

fn component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(ComponentOptionsRef {
            object: object.as_ref(),
        }),
        ExportDefaultDeclarationKind::CallExpression(call) => {
            component_options_from_call(call, bindings)
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(ComponentOptionsRef {
            object: object.as_ref(),
        }),
        Expression::CallExpression(call) => component_options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression, bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }

    let first_arg = call.arguments.first()?;
    component_options_from_argument(first_arg, bindings)
}

fn component_options_from_argument<'a>(
    argument: &'a Argument<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(ComponentOptionsRef {
            object: object.as_ref(),
        }),
        Argument::CallExpression(call) => component_options_from_call(call, bindings),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Argument::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression, bindings)
        }
        Argument::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression, bindings)
        }
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn object_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            object_expression_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            object_expression_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn object_expression_from_expression_or_binding<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::Identifier(identifier) => {
            object_bindings.get(identifier.name.as_str()).copied()
        }
        _ => object_expression_from_expression(expression),
    }
}

fn returned_object_from_expression<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()?
                else {
                    return None;
                };
                object_expression_from_expression_or_binding(&expr_stmt.expression, object_bindings)
            } else {
                function_body_return_object(&arrow.body.statements, object_bindings)
            }
        }
        Expression::FunctionExpression(function) => {
            function_body_return_object(&function.body.as_ref()?.statements, object_bindings)
        }
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::Identifier(identifier) => {
            object_bindings.get(identifier.name.as_str()).copied()
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            returned_object_from_expression(&parenthesized.expression, object_bindings)
        }
        Expression::TSAsExpression(ts_as) => {
            returned_object_from_expression(&ts_as.expression, object_bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            returned_object_from_expression(&ts_satisfies.expression, object_bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            returned_object_from_expression(&ts_non_null.expression, object_bindings)
        }
        _ => None,
    }
}

fn function_body_return_object<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in statements.iter() {
        let Statement::ReturnStatement(ret) = statement else {
            continue;
        };
        let Some(argument) = &ret.argument else {
            continue;
        };
        if let Some(object) =
            object_expression_from_expression_or_binding(argument, object_bindings)
        {
            return Some(object);
        }
    }

    None
}

fn local_name_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            local_name_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => local_name_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            local_name_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            local_name_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(callee.name.as_str(), "defineComponent" | "_defineComponent")
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        _ => false,
    }
}

fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        object_expression_from_expression_or_binding(&property.value, object_bindings)
    })
}

fn option_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        Some(&property.value)
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
