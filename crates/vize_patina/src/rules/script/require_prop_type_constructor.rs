//! script/require-prop-type-constructor
//!
//! Require Options API prop `type` values to be constructors rather than
//! literals or computed values.
//!
//! A prop's runtime `type` is matched against the constructor of the passed
//! value, so it must be the constructor *identifier* (`String`, `Number`,
//! `Boolean`, `Array`, `Object`, `Function`, `Symbol`, `Date`, ...). Writing the
//! type as a string literal (`type: "String"`) looks plausible but is wrong: Vue
//! compares the prop value's constructor against the string `"String"`, which
//! never matches, so the declared validation silently never fires.
//!
//! This is scoped to the Options API `props` option, in its object form. It
//! flags:
//!
//! * the shorthand where the prop value itself is the type
//!   (`count: "Number"`),
//! * an explicit `type` string literal (`count: { type: "Number" }`),
//! * string-literal entries inside a `type` array
//!   (`count: { type: ["Number", "String"] }`).
//!
//! Like the reference rule, the same three positions also reject every other
//! literal (`1`, `true`, a template literal), a binary expression such as the
//! common `Number | String` (a bitwise OR, which evaluates to `0`), and an update
//! expression. `null` stays valid: it declares a prop of any type.
//!
//! Constructor identifiers and any other expression (e.g. an imported type, a
//! `() => T` validator) are left alone. Array-form `props: ['a', 'b']` declare
//! only names — no types — so they are never flagged.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     // The type should be the `String` constructor, not the string "String".
//!     name: "String",
//!     age: { type: "Number" },
//!     id: { type: ["String", "Number"] }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     name: String,
//!     age: { type: Number },
//!     id: { type: [String, Number] }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, StringLiteral};
use oxc_span::{GetSpan, Span};
use vize_l0::CompactString;

mod options;
use options::{find_component_options, find_props_object, property_key_name};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-prop-type-constructor",
    description: "Require prop `type` values to be constructors rather than string literals",
    default_severity: Severity::Error,
};

/// Require Options API prop `type` values to be constructors, not string literals.
pub struct RequirePropTypeConstructor;

impl ScriptRule for RequirePropTypeConstructor {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(options) = find_component_options(program) else {
            return;
        };
        let Some(props) = find_props_object(options) else {
            return;
        };
        check_props_object(props, source, offset, result);
    }
}

/// Inspect each entry of the `props` object for a string-literal type.
fn check_props_object(
    props: &ObjectExpression<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        check_prop_type(&property.value, source, offset, result);
    }
}

/// Examine a single prop's value (the right-hand side of `name: <value>`).
///
/// In the Options API a prop value is one of:
/// * the type directly (`name: String` / the offending `name: "String"`),
/// * a descriptor object whose `type` carries the type
///   (`name: { type: String, required: true }`),
/// * an array shorthand of constructors (`name: [String, Number]`).
fn check_prop_type(
    value: &Expression<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    match value {
        // Descriptor object: inspect its `type` property, if any.
        Expression::ObjectExpression(object) => {
            if let Some(type_value) = find_type_value(object) {
                report_type_value(type_value, source, offset, result);
            }
        }
        // Shorthand: the prop value *is* the type, or an array of types.
        _ => report_type_value(value, source, offset, result),
    }
}

/// The expression bound to the `type` property of a prop descriptor object, if
/// present (`{ type: <value>, ... }`).
fn find_type_value<'a>(object: &'a ObjectExpression<'a>) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("type")) {
            return Some(&property.value);
        }
    }
    None
}

/// Report a type position that cannot be a constructor, or every such element
/// of a type array. Constructor identifiers, validators and `null` are fine.
fn report_type_value(
    type_value: &Expression<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    match type_value {
        Expression::StringLiteral(literal) => report(literal, offset, result),
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                match element.as_expression() {
                    Some(Expression::StringLiteral(literal)) => report(literal, offset, result),
                    Some(element) if is_non_constructor(element) => {
                        report_expression(element.span(), source, offset, result);
                    }
                    _ => {}
                }
            }
        }
        value if is_non_constructor(value) => {
            report_expression(value.span(), source, offset, result);
        }
        _ => {}
    }
}

/// The reference rule's forbidden node types besides string literals: the other
/// literals, template literals, binary expressions and update expressions. A
/// `null` literal is the valid "any type" declaration.
fn is_non_constructor(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::BinaryExpression(_)
            | Expression::UpdateExpression(_)
    )
}

/// Emit the diagnostic for an offending non-string type expression.
fn report_expression(span: Span, source: &str, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    let text = source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default();

    let mut message = CompactString::with_capacity(text.len() + 40);
    message.push_str("Prop type `");
    message.push_str(text);
    message.push_str("` should be a constructor.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("not a constructor", start, end)
        .with_help(
            "Use a constructor such as `String`, or an array of constructors such as `[Number, String]` to accept several types.",
        );
    result.add_diagnostic(diagnostic);
}

/// Emit the diagnostic for a single offending string-literal type.
fn report(literal: &StringLiteral<'_>, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + literal.span.start;
    let end = offset as u32 + literal.span.end;

    let type_name = literal.value.as_str();
    let mut message = CompactString::with_capacity(type_name.len() + 40);
    message.push_str("Prop type \"");
    message.push_str(type_name);
    message.push_str("\" should be a constructor, not a string.");

    let mut help = CompactString::with_capacity(type_name.len() + 40);
    help.push_str("Use the constructor `");
    help.push_str(type_name);
    help.push_str("` (without quotes) as the prop type.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("string literal used as a prop type", start, end)
        .with_help(help);
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
mod tests;
