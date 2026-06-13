//! script/require-typed-object-prop
//!
//! Require an explicit element type on a prop whose runtime type is the bare
//! `Object` / `Array` constructor.
//!
//! A runtime prop declared as `Object` or `Array` accepts any object / any
//! array — Vue can validate only the constructor, so the prop's value type
//! collapses to `Record<string, any>` / `any[]` and downstream code loses all
//! type-checking. Pairing the constructor with an `as PropType<T>` cast (or
//! switching to the type-based `defineProps<{ ... }>()` form) restores the
//! intended element type while keeping the runtime declaration.
//!
//! This flags an object-form prop whose runtime type is `Object` or `Array`
//! with no `as PropType<...>` cast, in either notation:
//!
//! * the shorthand where the prop value *is* the constructor
//!   (`foo: Object` / `foo: Array`), and
//! * an explicit `type` member (`foo: { type: Object }` /
//!   `foo: { type: Array }`).
//!
//! A cast (`Object as PropType<Foo>`, `Array as PropType<Bar[]>`) is the fix and
//! is left alone, as is any other constructor (`String`, `Number`, a custom
//! class), an array of constructors, an imported type, or a validator function.
//! The type-based `defineProps<{ ... }>()` form carries no runtime descriptor
//! and is never flagged. Covers the Options API `props` option (including
//! `defineComponent({...})` and same-file identifier-bound objects) and the
//! `<script setup>` runtime `defineProps(...)`.
//!
//! Mirrors [`vue/require-typed-object-prop`](https://eslint.vuejs.org/rules/require-typed-object-prop.html),
//! which applies to TypeScript only. All built-in script rules parse with
//! TypeScript semantics, so the `as PropType<...>` cast is observable.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const props = defineProps({
//!   foo: Object,
//!   bar: { type: Array },
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! const props = defineProps({
//!   foo: Object as PropType<Foo>,
//!   bar: { type: Array as PropType<Bar[]> },
//! })
//!
//! // Type-based form carries the element type directly.
//! const typed = defineProps<{ foo: Foo; bar: Bar[] }>()
//! ```

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey};
use oxc_span::GetSpan;

use vize_carton::CompactString;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::props_source::{PropDescriptor, collect_runtime_props};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-typed-object-prop",
    description: "Require an explicit type on a prop whose runtime type is `Object` or `Array`",
    default_severity: Severity::Warning,
};

/// Require `as PropType<T>` on a prop typed with the bare `Object`/`Array` constructor.
pub struct RequireTypedObjectProp;

impl ScriptRule for RequireTypedObjectProp {
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
        // Array-form props (`['foo']` / `defineProps(['foo'])`) declare only a
        // name and carry no runtime type, so there is nothing to inspect here;
        // a missing type is `script/require-prop-types`' concern.
        for source in collect_runtime_props(program) {
            for prop in source.object_props() {
                check_prop(prop, offset, result);
            }
        }
    }
}

/// Inspect a single object-form prop for a bare `Object`/`Array` runtime type.
fn check_prop(prop: PropDescriptor<'_>, offset: usize, result: &mut ScriptLintResult) {
    match prop.value {
        // Descriptor object: inspect its `type` member, if any.
        Expression::ObjectExpression(descriptor) => {
            if let Some(type_value) = find_type_value(descriptor) {
                check_type_expression(prop.name, type_value, offset, result);
            }
        }
        // Shorthand: the prop value *is* the type (`foo: Object`).
        value => check_type_expression(prop.name, value, offset, result),
    }
}

/// Report a `type` expression that is the bare `Object`/`Array` constructor.
///
/// An `as PropType<...>` cast (a [`TSAsExpression`]) is the fix, so a cast value
/// is intentionally *not* flagged regardless of what it wraps. Every other
/// expression — a different constructor, an array of constructors, an imported
/// type, a validator — carries (or is) a type and is left alone.
///
/// [`TSAsExpression`]: oxc_ast::ast::Expression::TSAsExpression
fn check_type_expression(
    name: &str,
    type_value: &Expression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if let Expression::Identifier(identifier) = type_value
        && let Some(constructor) = bare_object_or_array(identifier.name.as_str())
    {
        report(name, constructor, type_value.span(), offset, result);
    }
}

/// The `Object` / `Array` constructor name, or `None` for any other identifier.
fn bare_object_or_array(name: &str) -> Option<&'static str> {
    match name {
        "Object" => Some("Object"),
        "Array" => Some("Array"),
        _ => None,
    }
}

/// The expression bound to the `type` member of a prop descriptor object, if
/// present (`{ type: <value>, ... }`). Computed keys are skipped.
fn find_type_value<'a>(descriptor: &'a ObjectExpression<'a>) -> Option<&'a Expression<'a>> {
    descriptor.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some("type") {
            return None;
        }
        Some(&property.value)
    })
}

fn report(
    name: &str,
    constructor: &str,
    span: oxc_span::Span,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let mut message = CompactString::with_capacity(name.len() + constructor.len() + 48);
    message.push_str("Prop '");
    message.push_str(name);
    message.push_str("' typed as `");
    message.push_str(constructor);
    message.push_str("` should have an explicit element type.");

    let mut help = CompactString::with_capacity(constructor.len() + 64);
    help.push_str("Add an `as PropType<T>` cast, e.g. `");
    help.push_str(constructor);
    help.push_str(" as PropType<T>`, or use the type-based `defineProps<{ ... }>()` form.");

    let diagnostic = LintDiagnostic::warn(META.name, message, start, end)
        .with_label(
            "bare `Object`/`Array` constructor used as a prop type",
            start,
            end,
        )
        .with_help(help);
    result.add_diagnostic(diagnostic);
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
