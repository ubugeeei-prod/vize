//! Options API component metadata collection.
//!
//! Walks `export default { ... }` / `defineComponent({ ... })` options objects
//! to collect component registrations and template bindings (props, data,
//! computed, methods, inject, setup, same-file mixins/extends) for the
//! Options API and legacy Vue 2.7.

use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, BindingPattern, CallExpression,
    ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey, Statement,
};
use oxc_span::GetSpan;

use crate::ScopeBinding;
use crate::croquis::ComponentRegistration;
use vize_carton::{CompactString, FxHashMap, FxHashSet, String};
use vize_relief::BindingType;

use super::super::ScriptParseResult;

#[derive(Clone, Copy)]
struct ComponentOptionsRef<'a> {
    object: &'a ObjectExpression<'a>,
}

pub(in crate::script_parser) fn collect_options_api_component_metadata(
    result: &mut ScriptParseResult,
    program: &Program<'_>,
    options_api: bool,
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

        // Class components (vue-class-component / vue-property-decorator):
        // in an SFC the default export *is* the component, so a class default
        // export is unambiguous. Auto-detected by AST shape — no flag, and
        // this arm never executes for non-class components.
        if let Some(class) = super::class_component::class_from_export(&export.declaration) {
            super::class_component::collect_class_component_metadata(
                result,
                class,
                &object_bindings,
            );
            continue;
        }

        let Some(options) =
            component_options_from_export(&export.declaration, &component_option_bindings)
        else {
            continue;
        };
        collect_component_registrations_from_options(result, options.object, &object_bindings);
        // Options API template bindings are valid in Vue 3 too; legacy Vue 2.7
        // implies them and additionally pulls in the Nuxt 2 globals below.
        if options_api || legacy_vue2 {
            collect_options_api_template_bindings_from_options(
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

pub(super) fn collect_component_registrations_from_options<'a>(
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

pub(super) fn collect_options_api_template_bindings_from_options<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    let mut seen_mixins = FxHashSet::default();
    collect_options_object_template_bindings(result, options, object_bindings, &mut seen_mixins);
}

/// Collects Options API template bindings from a single options
/// `ObjectExpression`, recursing into same-file `mixins`/`extends` targets.
///
/// Deliberately dialect-agnostic: this helper only requires a plain
/// `ObjectExpression`, so the upcoming legacy-Vue work (`Vue.extend({...})`
/// callee recognition, issue #1392) can reuse it by unwrapping the call
/// expression to its options-object argument before recursing.
fn collect_options_object_template_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
) {
    collect_array_or_object_option_bindings(
        result,
        options,
        object_bindings,
        "props",
        BindingType::Props,
    );
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
    // `inject` accepts both the array form (`inject: ['foo']`) and the object
    // form, so it is routed through the array-or-object collector like props.
    collect_array_or_object_option_bindings(
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
    // Options API `setup()` return values are setup bindings, not `options`
    // members: `@vue/compiler-sfc` types them `setup-maybe-ref`, so the template
    // compiler prefixes them with `$setup.` (not `$options.`) in non-inline mode.
    collect_returned_object_option_bindings(
        result,
        options,
        object_bindings,
        "setup",
        BindingType::SetupMaybeRef,
    );
    collect_mixins_bindings(result, options, object_bindings, seen_mixins);
    collect_extends_bindings(result, options, object_bindings, seen_mixins);
}

/// Merges template bindings contributed by same-file `mixins` entries.
///
/// Only same-file targets are resolved: inline object literals and
/// identifiers whose `const` initializer is an object literal in this module.
/// Imported mixins are deliberately ignored — resolving them requires
/// cross-file analysis, which is deferred.
fn collect_mixins_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
) {
    let Some(Expression::ArrayExpression(array)) = option_expression_property(options, "mixins")
    else {
        return;
    };

    for element in &array.elements {
        let Some(expression) = element.as_expression() else {
            continue;
        };
        collect_mixin_target_bindings(result, expression, object_bindings, seen_mixins);
    }
}

/// Merges template bindings contributed by a same-file `extends` target.
/// Same resolution rules and deferral as [`collect_mixins_bindings`].
fn collect_extends_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
) {
    let Some(expression) = option_expression_property(options, "extends") else {
        return;
    };
    collect_mixin_target_bindings(result, expression, object_bindings, seen_mixins);
}

/// Resolves a single mixin/extends target expression and merges its option
/// bindings. The seen-set guards against mixin cycles (A mixes B mixes A);
/// inline object literals cannot cycle because the AST is a tree.
fn collect_mixin_target_bindings<'a>(
    result: &mut ScriptParseResult,
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
) {
    match expression {
        Expression::ObjectExpression(object) => {
            collect_options_object_template_bindings(result, object, object_bindings, seen_mixins);
        }
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if !seen_mixins.insert(name) {
                return;
            }
            if let Some(object) = object_bindings.get(name).copied() {
                collect_options_object_template_bindings(
                    result,
                    object,
                    object_bindings,
                    seen_mixins,
                );
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => collect_mixin_target_bindings(
            result,
            &parenthesized.expression,
            object_bindings,
            seen_mixins,
        ),
        Expression::TSAsExpression(ts_as) => {
            collect_mixin_target_bindings(result, &ts_as.expression, object_bindings, seen_mixins)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => collect_mixin_target_bindings(
            result,
            &ts_satisfies.expression,
            object_bindings,
            seen_mixins,
        ),
        Expression::TSNonNullExpression(ts_non_null) => collect_mixin_target_bindings(
            result,
            &ts_non_null.expression,
            object_bindings,
            seen_mixins,
        ),
        // Imported mixins, call expressions, etc. — deferred.
        _ => {}
    }
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

/// Collects bindings from an option whose value may be an array of string
/// literals (`['foo', 'bar']`), an object literal keyed by binding name, or a
/// same-file identifier resolving to such an object. Used by `props` and
/// `inject`, which both accept the array and object forms.
fn collect_array_or_object_option_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    key_name: &str,
    binding_type: BindingType,
) {
    let Some(expression) = option_expression_property(options, key_name) else {
        return;
    };
    collect_array_or_object_bindings_from_expression(
        result,
        expression,
        object_bindings,
        binding_type,
    );
}

fn collect_array_or_object_bindings_from_expression<'a>(
    result: &mut ScriptParseResult,
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    binding_type: BindingType,
) {
    match expression {
        Expression::ArrayExpression(array) => {
            collect_array_string_bindings(result, array, binding_type);
        }
        Expression::ObjectExpression(object) => {
            collect_object_property_bindings(result, object, binding_type);
        }
        Expression::Identifier(identifier) => {
            if let Some(object) = object_bindings.get(identifier.name.as_str()).copied() {
                collect_object_property_bindings(result, object, binding_type);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_array_or_object_bindings_from_expression(
                result,
                &parenthesized.expression,
                object_bindings,
                binding_type,
            )
        }
        Expression::TSAsExpression(ts_as) => collect_array_or_object_bindings_from_expression(
            result,
            &ts_as.expression,
            object_bindings,
            binding_type,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_array_or_object_bindings_from_expression(
                result,
                &ts_satisfies.expression,
                object_bindings,
                binding_type,
            )
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_array_or_object_bindings_from_expression(
                result,
                &ts_non_null.expression,
                object_bindings,
                binding_type,
            )
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

pub(super) fn add_template_binding(
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

pub(super) fn normalize_template_binding_name(name: &str) -> Option<CompactString> {
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

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}
