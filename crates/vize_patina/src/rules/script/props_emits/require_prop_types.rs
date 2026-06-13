//! script/require-prop-types
//!
//! Require every prop to declare a type.
//!
//! A typed prop documents its contract and lets Vue validate values it receives
//! at runtime. A prop declared with no type accepts anything, so the
//! declaration provides neither documentation nor a runtime guard.
//!
//! This flags a prop that carries no type:
//! * an array-form entry (`props: ['foo']` / `defineProps(['foo'])`), which by
//!   construction declares only a name, and
//! * an object-form prop with no `type` member, or whose value is `null`
//!   (`foo: null`) or an empty descriptor (`foo: {}`).
//!
//! An object-form prop whose value is itself the type (`foo: String`,
//! `foo: [String, Number]`) or whose descriptor has a `type` member is typed
//! and left alone. Covers the Options API `props` option (including
//! `defineComponent({...})` and same-file identifier-bound objects) and the
//! `<script setup>` runtime `defineProps(...)`.
//!
//! Port of [`vue/require-prop-types`](https://eslint.vuejs.org/rules/require-prop-types.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: ['status']            // array form: no types
//! }
//!
//! export default {
//!   props: {
//!     status: null,              // no type
//!     other: {}                  // empty descriptor: no type
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     status: String,
//!     other: { type: Number, default: 0 }
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
    name: "script/require-prop-types",
    description: "Require every prop to declare a type",
    default_severity: Severity::Error,
};

/// Require every prop to declare a type.
pub struct RequirePropTypes;

impl ScriptRule for RequirePropTypes {
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
            // Array-form entries declare only a name — always untyped.
            for literal in source.array_names() {
                report(literal.value.as_str(), literal.span, offset, result);
            }
            // Object-form entries are typed unless their value is `null` / an
            // empty-or-typeless descriptor.
            for prop in source.object_props() {
                if !prop_has_type(prop) {
                    report(prop.name, prop.key.span(), offset, result);
                }
            }
        }
    }
}

/// Whether an object-form prop declares a type.
///
/// A prop is typed when its value is the type directly (`foo: String`,
/// `foo: [String]`, an imported type identifier, a `() => T` validator) or a
/// descriptor object carrying a `type` member. It is untyped when the value is
/// `null` or a descriptor with no `type` member (including `{}`).
fn prop_has_type(prop: PropDescriptor<'_>) -> bool {
    match prop.value {
        // `foo: null` declares no type.
        Expression::NullLiteral(_) => false,
        // A descriptor object is typed only if it has a `type` member.
        Expression::ObjectExpression(descriptor) => descriptor_has_type(descriptor),
        // Any other value (constructor, array of constructors, validator,
        // imported type) is itself the type.
        _ => true,
    }
}

/// Whether a prop-descriptor object has a `type` member.
fn descriptor_has_type(descriptor: &ObjectExpression<'_>) -> bool {
    descriptor.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        !property.computed && property_key_name(&property.key) == Some("type")
    })
}

fn report(name: &str, span: oxc_span::Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let mut message = CompactString::with_capacity(name.len() + 36);
    message.push_str("Prop '");
    message.push_str(name);
    message.push_str("' should define a type.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("prop declared without a type", start, end)
        .with_help(
            "Declare the prop's type, e.g. `{ type: String }` or the shorthand `status: String` \
             instead of the array form or a typeless descriptor.",
        );
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
