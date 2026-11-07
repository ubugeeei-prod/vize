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
//! * an object-form prop with an empty array, function value, or descriptor
//!   carrying neither `type` nor `validator`.
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

use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

use vize_l0::CompactString;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::compiler_macro_is_shadowed;
use super::props_source::{PropDescriptor, collect_all_runtime_props};
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
        for source in collect_all_runtime_props(program) {
            // Array-form entries declare only a name — always untyped.
            for literal in source.array_names() {
                report(literal.value.as_str(), literal.span, offset, result);
            }
            // Object-form entries are typed unless their value is `null` / an
            // empty-or-typeless descriptor.
            for prop in source.object_props() {
                if !prop_has_type(prop) {
                    report(prop.name, prop.span, offset, result);
                }
            }
        }
        if !compiler_macro_is_shadowed(program, "defineModel") {
            let mut visitor = DefineModelVisitor { offset, result };
            visitor.visit_program(program);
        }
    }
}

struct DefineModelVisitor<'out> {
    offset: usize,
    result: &'out mut ScriptLintResult,
}

impl<'a> Visit<'a> for DefineModelVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "defineModel")
            && call.type_arguments.is_none()
        {
            check_define_model(call, self.offset, self.result);
        }
        walk_call_expression(self, call);
    }
}

fn check_define_model(call: &CallExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    let mut name = "modelValue";
    let mut options_index = 0;
    if let Some(Argument::StringLiteral(literal)) = call.arguments.first() {
        name = literal.value.as_str();
        options_index = 1;
    }
    let Some(options) = call
        .arguments
        .get(options_index)
        .and_then(Argument::as_expression)
    else {
        report(name, call.span, offset, result);
        return;
    };
    let Expression::ObjectExpression(options) = options.get_inner_expression() else {
        // An unresolved expression may provide a type. eslint-plugin-vue does
        // not report when it cannot prove the options are typeless.
        return;
    };
    if !descriptor_has_type(options) {
        report(name, options.span, offset, result);
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
        // eslint-plugin-vue treats null and unknown expressions as intentionally
        // unconstrained rather than incomplete declarations.
        Expression::NullLiteral(_) => true,
        // A descriptor object is typed only if it has a `type` member.
        Expression::ObjectExpression(descriptor) => descriptor_has_type(descriptor),
        Expression::ArrayExpression(types) => !types.elements.is_empty(),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => false,
        // Any other value (constructor, array of constructors, validator,
        // imported type) is itself the type.
        _ => true,
    }
}

/// Whether a prop-descriptor object has a non-empty `type` member or a
/// `validator`, matching eslint-plugin-vue's default contract.
fn descriptor_has_type(descriptor: &ObjectExpression<'_>) -> bool {
    descriptor.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if property.computed {
            return false;
        }
        match property_key_name(&property.key) {
            Some("validator") => true,
            Some("type") => !matches!(
                &property.value,
                Expression::ArrayExpression(types) if types.elements.is_empty()
            ),
            _ => false,
        }
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
