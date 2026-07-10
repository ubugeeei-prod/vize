//! script/require-valid-default-prop
//!
//! Require a prop's `default` value to be valid for its declared runtime type.
//!
//! Vue applies a prop's `default` when the parent omits the prop. The default
//! must agree with the declared `type`, otherwise the component renders a value
//! the type contract forbids:
//!
//! * a scalar type with a mismatched literal — `type: Number, default: 'x'`,
//!   `type: Boolean, default: 0` — silently feeds the wrong kind of value, and
//! * an `Object` / `Array` type with a literal default — `default: {}`,
//!   `default: []` — shares one mutable instance across every component using
//!   the component, so Vue requires a **factory function**
//!   (`default: () => ({})`) instead.
//!
//! A function default is always accepted: it is the factory Vue calls per
//! instance, and its return value cannot be checked statically. A default whose
//! kind cannot be determined (an identifier, a call expression, …) is left
//! alone. A prop whose declared `type` is not one of the native constructors
//! (`String`, `Number`, `Boolean`, `Array`, `Object`, `Function`, `Symbol`,
//! `BigInt`) — e.g. an imported `PropType` or a custom class — is also skipped,
//! since its valid defaults cannot be known.
//!
//! Covers the Options API object form (`props: { x: { type, default } }`),
//! including `defineComponent({...})` and same-file identifier-bound
//! options/props objects, and the `<script setup>` runtime form
//! `defineProps({ ... })`. The array shorthand and the type-based
//! `defineProps<{...}>()` form carry no runtime `default` and are not checked.
//!
//! Port of [`vue/require-valid-default-prop`](https://eslint.vuejs.org/rules/require-valid-default-prop.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     count: { type: Number, default: '0' },     // string default for Number
//!     enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
//!     items: { type: Array, default: [] },        // literal must be a factory
//!     config: { type: Object, default: {} }       // literal must be a factory
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     count: { type: Number, default: 0 },
//!     enabled: { type: Boolean, default: false },
//!     items: { type: Array, default: () => [] },
//!     config: { type: Object, default: () => ({}) },
//!     label: { type: [String, Number], default: '' }
//!   }
//! }
//! ```

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey};
use oxc_span::GetSpan;

use vize_carton::CompactString;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::props_source::{PropDescriptor, collect_runtime_props};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-valid-default-prop",
    description: "Require a prop's default value to be valid for its declared type",
    default_severity: Severity::Error,
};

/// The native constructor types Vue recognizes for runtime prop validation.
const NATIVE_TYPES: [&str; 8] = [
    "String", "Number", "Boolean", "Function", "Object", "Array", "Symbol", "BigInt",
];

/// Declared types whose only acceptable default is a factory function: a literal
/// `Object`/`Array` would be shared across instances, and a `Function` prop's
/// value is the function itself.
const FUNCTION_VALUE_TYPES: [&str; 3] = ["Function", "Object", "Array"];

/// Require a prop's `default` to be valid for its declared type.
pub struct RequireValidDefaultProp;

impl ScriptRule for RequireValidDefaultProp {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for source in collect_runtime_props(program) {
            for prop in source.object_props() {
                check_prop(prop, offset, result);
            }
        }
    }
}

/// The value-type a default expression denotes, mirroring Vue's runtime
/// `typeof`/constructor check. `None` means the kind cannot be determined
/// statically (an identifier, a call, …), so no validation is attempted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DefaultKind {
    /// A literal of a native scalar/reference type (`"x"`, `0`, `true`, `[]`,
    /// `{}`, `1n`).
    Value(&'static str),
    /// A function (`function () {}` / `() => …`): the factory Vue calls.
    Function,
}

/// Validate the `default` of a single object-descriptor prop against its `type`.
fn check_prop(prop: PropDescriptor<'_>, offset: usize, result: &mut ScriptLintResult) {
    // Only an object descriptor can carry both `type` and `default`. A shorthand
    // value (`name: String`) declares no default, so there is nothing to check.
    let Expression::ObjectExpression(descriptor) = prop.value else {
        return;
    };
    let Some(default_value) = descriptor_member(descriptor, "default") else {
        return;
    };
    let Some(type_value) = descriptor_member(descriptor, "type") else {
        return;
    };

    // The declared native types; non-native types (imported `PropType`, custom
    // classes) cannot be validated and are skipped.
    let type_names = native_type_names(type_value);
    if type_names.is_empty() {
        return;
    }

    let Some(default_kind) = default_kind(default_value) else {
        // Identifier, call expression, etc. — kind unknown, do not guess.
        return;
    };

    match default_kind {
        // A function default is the factory Vue invokes per instance; accept it
        // for every declared type (its return value is not checked statically).
        DefaultKind::Function => {}
        DefaultKind::Value(value_type) => {
            if type_names.contains(&value_type) && !FUNCTION_VALUE_TYPES.contains(&value_type) {
                // A scalar literal matching a declared scalar type is valid.
                return;
            }
            // Either a type mismatch, or an `Object`/`Array` literal that must be
            // a factory function. The expected-type wording maps the
            // factory-only types to "function".
            report(prop.name, default_value, &type_names, offset, result);
        }
    }
}

/// The default value's [`DefaultKind`], or `None` if it cannot be determined.
fn default_kind(expression: &Expression<'_>) -> Option<DefaultKind> {
    match expression {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
            Some(DefaultKind::Value("String"))
        }
        Expression::NumericLiteral(_) => Some(DefaultKind::Value("Number")),
        Expression::BooleanLiteral(_) => Some(DefaultKind::Value("Boolean")),
        Expression::BigIntLiteral(_) => Some(DefaultKind::Value("BigInt")),
        Expression::ArrayExpression(_) => Some(DefaultKind::Value("Array")),
        Expression::ObjectExpression(_) => Some(DefaultKind::Value("Object")),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
            Some(DefaultKind::Function)
        }
        _ => None,
    }
}

/// The declared native constructor types of a `type` value: a single
/// constructor (`Number`) or the native entries of a `[...]` union
/// (`[String, Number]`). Non-native and non-identifier entries are dropped.
fn native_type_names(type_value: &Expression<'_>) -> Vec<&'static str> {
    match type_value {
        Expression::Identifier(identifier) => {
            native_type(identifier.name.as_str()).into_iter().collect()
        }
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .filter_map(|element| match element.as_expression() {
                Some(Expression::Identifier(identifier)) => native_type(identifier.name.as_str()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The canonical `'static` spelling of a native constructor name, or `None` if
/// `name` is not one of Vue's native prop types.
fn native_type(name: &str) -> Option<&'static str> {
    NATIVE_TYPES.into_iter().find(|native| *native == name)
}

/// The value of a named member of a prop-descriptor object, if present.
fn descriptor_member<'a>(
    descriptor: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<&'a Expression<'a>> {
    descriptor.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key) {
            return None;
        }
        Some(&property.value)
    })
}

/// Emit the diagnostic for an invalid default, naming the expected type(s).
///
/// Factory-only declared types (`Object`/`Array`/`Function`) are reported as
/// `function`, matching Vue's requirement that they use a factory; other types
/// are listed verbatim. The set is joined with " or " and lowercased.
fn report(
    name: &str,
    default_value: &Expression<'_>,
    type_names: &[&'static str],
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let span = default_value.span();
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let expected = expected_types(type_names);

    let mut message = CompactString::with_capacity(name.len() + expected.len() + 48);
    message.push_str("Type of the default value for '");
    message.push_str(name);
    message.push_str("' prop must be a ");
    message.push_str(&expected);
    message.push('.');

    let mut help = CompactString::with_capacity(expected.len() + 56);
    help.push_str("Make the default a ");
    help.push_str(&expected);
    help.push_str(" (use a factory like `() => ({})` for Object/Array props).");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("default does not match the declared type", start, end)
        .with_help(help);
    result.add_diagnostic(diagnostic);
}

/// The expected-type wording: declared types with the factory-only ones mapped
/// to `function`, joined with " or " and lowercased (`"string or number"`,
/// `"function"`).
fn expected_types(type_names: &[&'static str]) -> CompactString {
    let mut seen: Vec<&str> = Vec::with_capacity(type_names.len());
    for name in type_names {
        let mapped = if FUNCTION_VALUE_TYPES.contains(name) {
            "function"
        } else {
            *name
        };
        if !seen.contains(&mapped) {
            seen.push(mapped);
        }
    }

    let mut joined = CompactString::default();
    for (index, name) in seen.iter().enumerate() {
        if index > 0 {
            joined.push_str(" or ");
        }
        joined.push_str(name);
    }
    joined.to_lowercase()
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
