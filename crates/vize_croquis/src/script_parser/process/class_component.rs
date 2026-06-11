//! Class component (vue-class-component / vue-property-decorator) extraction.
//!
//! In an SFC the default export *is* the component, so a class default export
//! is unambiguous: it is a class component. Detection is purely shape-based
//! (a match arm on the export-default AST node) — no configuration flag, and
//! zero added work for non-class components.
//!
//! Class members map onto the same binding model the Options API produces:
//!
//! | Class member                  | Binding type | Options API equivalent |
//! |-------------------------------|--------------|------------------------|
//! | `@Prop`-style field/accessor  | `Props`      | `props`                |
//! | other property field/accessor | `Data`       | `data()`               |
//! | method                        | `Options`    | `methods`              |
//! | `get` / `set` accessor        | `Options`    | `computed`             |
//!
//! ## Visibility
//!
//! TS accessibility modifiers (`private` / `protected`) are erased at runtime,
//! and the canonical vue-class-component scaffold relies on that (e.g. the Vue
//! CLI template renders `{{ msg }}` from `@Prop() private msg!: string`), so
//! TS-private members are still extracted as template bindings: the template
//! *can* resolve them at runtime. Type-level visibility enforcement is the
//! virtual-TS checker's job once canon bridges the class instance type.
//! ECMAScript hard-private members (`#name`) are genuinely unreachable outside
//! the class body and are skipped, along with `static` members (not on the
//! instance), `declare` fields, computed keys, and the constructor.
//!
//! Member-level decorators are interpreted as follows:
//!
//! - `@Prop`-style decorators (`@Prop`/`@PropSync`/`@Model`/`@VModel`) map a
//!   field/accessor to a prop binding (see #1431).
//! - `@Emit('name')` on a method declares an emitted event, mirroring an
//!   Options API `emits` entry / a `defineEmits` declaration: the event name
//!   defaults to the method name kebab-cased when no string argument is given.
//! - `@Inject`-style decorators (`@Inject`/`@InjectReactive`) mark a field as
//!   an injected binding, resolved like an Options API `inject` member
//!   (`BindingType::Options`) rather than reactive `data`.
//!
//! `@Watch` (watcher registration) and `@Provide` have no direct template /
//! type-resolution effect — the watched/provided members keep their ordinary
//! binding classification — so they are deliberately not interpreted here.
//!
//! The `@Component({ ... })` / `@Options({ ... })` decorator argument is a
//! regular options object and is fed through the existing Options API
//! collectors (so `components:` registrations and inline
//! `data`/`computed`/`methods` behave identically to an options component).

use oxc_ast::ast::{
    Argument, Class, ClassElement, Decorator, ExportDefaultDeclarationKind, Expression,
    MethodDefinition, MethodDefinitionKind, ObjectExpression, PropertyKey,
};
use oxc_span::GetSpan;

use vize_carton::{CompactString, FxHashMap, hyphenate};
use vize_relief::BindingType;

use super::super::ScriptParseResult;
use super::options_api::{
    add_template_binding, collect_component_registrations_from_options,
    collect_options_api_template_bindings_from_options, normalize_template_binding_name,
    property_key_name,
};
use crate::croquis::ComponentShape;
use crate::macros::EmitDefinition;

/// Return the class when the default export is a class declaration or a
/// (possibly TS-wrapped / parenthesized) class expression.
pub(super) fn class_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a Class<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ClassDeclaration(class) => Some(class.as_ref()),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            class_from_expression(&parenthesized.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            class_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            class_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            class_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn class_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a Class<'a>> {
    match expression {
        Expression::ClassExpression(class) => Some(class.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            class_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => class_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            class_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            class_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

/// Extract template bindings and component registrations from a class
/// component's members and its `@Component` / `@Options` decorator argument.
pub(super) fn collect_class_component_metadata<'a>(
    result: &mut ScriptParseResult,
    class: &'a Class<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    result.component_shape = ComponentShape::ClassApi;

    // `@Component({ ... })` / `@Options({ ... })` take a regular options
    // object; reuse the Options API collectors so it behaves identically to
    // an options component.
    for decorator in &class.decorators {
        let Some(options) = decorator_options_object(&decorator.expression) else {
            continue;
        };
        collect_component_registrations_from_options(result, options, object_bindings);
        collect_options_api_template_bindings_from_options(result, options, object_bindings);
    }

    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                if method.r#static
                    || method.computed
                    || matches!(method.kind, MethodDefinitionKind::Constructor)
                {
                    continue;
                }
                // `@Emit` on a method declares an emitted event (the method
                // body still runs and its return value is the payload).
                collect_emit_decorator(result, method);
                // Methods map to `methods`, get/set accessors to `computed`;
                // both resolve as `Options` bindings.
                add_class_member_binding(result, &method.key, BindingType::Options);
            }
            ClassElement::PropertyDefinition(property) => {
                if property.r#static || property.computed || property.declare {
                    continue;
                }
                add_class_member_binding(
                    result,
                    &property.key,
                    class_field_binding_type(&property.decorators),
                );
            }
            ClassElement::AccessorProperty(accessor) => {
                if accessor.r#static || accessor.computed {
                    continue;
                }
                add_class_member_binding(
                    result,
                    &accessor.key,
                    class_field_binding_type(&accessor.decorators),
                );
            }
            ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => {}
        }
    }
}

fn class_field_binding_type(decorators: &[Decorator<'_>]) -> BindingType {
    // `@Prop`-style decorators win over `@Inject` (a field is never both), and
    // a plain field falls back to reactive `data`.
    if decorators
        .iter()
        .any(|decorator| member_decorator_name(decorator).is_some_and(is_prop_like_decorator_name))
    {
        BindingType::Props
    } else if decorators
        .iter()
        .any(|decorator| member_decorator_name(decorator).is_some_and(is_inject_decorator_name))
    {
        // `@Inject`/`@InjectReactive` members are injected bindings, resolved
        // like an Options API `inject` entry (`BindingType::Options`).
        BindingType::Options
    } else {
        BindingType::Data
    }
}

/// Identifier name of a member decorator, unwrapping a call form
/// (`@Emit('x')`) to its callee. Returns `None` for non-identifier callees.
fn member_decorator_name<'a>(decorator: &'a Decorator<'a>) -> Option<&'a str> {
    match &decorator.expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(identifier) => Some(identifier.name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn is_prop_like_decorator_name(name: &str) -> bool {
    matches!(name, "Prop" | "PropSync" | "Model" | "ModelSync" | "VModel")
}

fn is_inject_decorator_name(name: &str) -> bool {
    matches!(name, "Inject" | "InjectReactive")
}

/// Record an `@Emit(...)` decorator on a method as an emitted event.
///
/// vue-property-decorator's `@Emit` emits the event named by the decorator's
/// string argument, defaulting to the method name kebab-cased when omitted
/// (`@Emit() onReset()` -> `on-reset`). The payload type is left unresolved
/// here: the method body is a real method whose return value is the payload,
/// so payload typing is the virtual-TS bridge's job, not the binding pass.
fn collect_emit_decorator(result: &mut ScriptParseResult, method: &MethodDefinition<'_>) {
    for decorator in &method.decorators {
        if member_decorator_name(decorator) != Some("Emit") {
            continue;
        }

        let name = match emit_decorator_event_name(&decorator.expression) {
            Some(name) => name,
            None => {
                let Some(method_name) = property_key_name(&method.key) else {
                    continue;
                };
                CompactString::new(hyphenate(method_name))
            }
        };

        result.macros.add_emit(EmitDefinition {
            name,
            payload_type: None,
        });
        // A method carries at most one meaningful `@Emit`; stop after the first.
        break;
    }
}

/// The explicit string-literal event name from `@Emit('name')`, if any.
fn emit_decorator_event_name(expression: &Expression<'_>) -> Option<CompactString> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    match call.arguments.first()? {
        Argument::StringLiteral(literal) => Some(CompactString::new(literal.value.as_str())),
        _ => None,
    }
}

/// Options object passed to a `@Component(...)` / `@Options(...)` decorator.
fn decorator_options_object<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "Component" | "Options") {
        return None;
    }
    let Some(Argument::ObjectExpression(object)) = call.arguments.first() else {
        return None;
    };
    Some(object.as_ref())
}

fn add_class_member_binding(
    result: &mut ScriptParseResult,
    key: &PropertyKey<'_>,
    binding_type: BindingType,
) {
    // `property_key_name` only resolves static identifier / string-literal
    // keys, so ECMAScript hard-private members (`#name`) drop out here.
    let Some(raw_name) = property_key_name(key) else {
        return;
    };
    let Some(name) = normalize_template_binding_name(raw_name) else {
        return;
    };
    let span = key.span();
    add_template_binding(result, name.as_str(), binding_type, span.start, span.end);
}
