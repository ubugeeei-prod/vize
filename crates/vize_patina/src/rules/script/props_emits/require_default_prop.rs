//! script/require-default-prop
//!
//! Require a `default` value for every optional, non-Boolean prop.
//!
//! A prop that is not `required: true` may be omitted by the parent. Unless it
//! declares a `default`, an omitted prop is `undefined`, which usually forces
//! defensive `?.` / `?? fallback` handling at every use site. Declaring a
//! `default` documents the intended fallback in one place.
//!
//! Two props never need a default and are skipped:
//! * a `required: true` prop (always provided by the parent), and
//! * a `Boolean`-typed prop (an absent boolean prop already defaults to
//!   `false`).
//!
//! Covers the Options API object form (`props: { x: { type: ... } }`),
//! including `defineComponent({...})` and same-file identifier-bound
//! options/props objects, and the `<script setup>` runtime form
//! `defineProps({ ... })`. The array shorthand (`props: ['x']` /
//! `defineProps(['x'])`) declares neither type nor default and is left to
//! `require-prop-types`. Type-based props are checked when they use a defaults
//! surface (`withDefaults` or props destructuring), including a local generic
//! interface declared in the SFC's sibling `<script>` block. A plain
//! `defineProps<T>()` has no authored default surface and is left alone,
//! matching eslint-plugin-vue.
//!
//! Port of [`vue/require-default-prop`](https://eslint.vuejs.org/rules/require-default-prop.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     // optional, non-Boolean, no default
//!     name: String,
//!     age: { type: Number },
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     name: { type: String, default: '' },
//!     enabled: Boolean,                 // Boolean defaults to false
//!     id: { type: Number, required: true },
//!   }
//! }
//! ```

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey};
use oxc_span::Span;

use vize_l0::CompactString;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use super::props_source::{PropDescriptor, PropsSource, collect_options_props};
use super::typed_prop_defaults::collect_setup_props_with_types;
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-default-prop",
    description: "Require a default value for every optional, non-Boolean prop",
    default_severity: Severity::Error,
};

/// Require a `default` for optional, non-Boolean props.
pub struct RequireDefaultProp;

impl ScriptRule for RequireDefaultProp {
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
        self.check_program_with_sfc(
            program,
            _source,
            offset,
            SfcScriptContext::default(),
            result,
        );
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if let Some(source) = collect_options_props(program) {
            for prop in source.object_props() {
                check_prop(prop, false, offset, result);
            }
        }

        let sibling = sfc.sibling_program.zip(sfc.sibling_offset);
        for declaration in collect_setup_props_with_types(program, offset, sibling) {
            if let Some(runtime) = declaration.runtime {
                let source = PropsSource::Object(runtime);
                for prop in source.object_props() {
                    check_prop(
                        prop,
                        declaration.uses_destructure && declaration.has_default(prop.name),
                        offset,
                        result,
                    );
                }
            }
            if declaration.uses_destructure || declaration.uses_with_defaults {
                for prop in &declaration.typed {
                    if prop.required || prop.boolean || declaration.has_default(&prop.name) {
                        continue;
                    }
                    report(&prop.name, prop.span, prop.offset, result);
                }
            }
        }
    }
}

/// Flag an object-descriptor prop that is optional, not Boolean-typed, and
/// declares no `default`.
fn check_prop(
    prop: PropDescriptor<'_>,
    has_destructure_default: bool,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if prop.shorthand || has_destructure_default {
        return;
    }
    // Only object-descriptor props can carry `required`/`default`/`type`. A
    // shorthand prop value (`name: String`) is by definition optional with no
    // default, so it is flagged too — unless its shorthand type is Boolean.
    match prop.value.get_inner_expression() {
        Expression::ObjectExpression(descriptor) => {
            if descriptor_required_true(descriptor)
                || descriptor_has_default(descriptor)
                || descriptor_type_is_boolean(descriptor)
            {
                return;
            }
            report(prop.name, prop.span, offset, result);
        }
        Expression::Identifier(identifier) => {
            if identifier.name == "Boolean" {
                return;
            }
            if matches!(
                identifier.name.as_str(),
                "String" | "Number" | "Function" | "Object" | "Array" | "Symbol"
            ) {
                report(prop.name, prop.span, offset, result);
            }
        }
        // Factory calls and member references are custom prop constructors;
        // eslint-plugin-vue cannot prove that they need a default.
        Expression::CallExpression(_) | Expression::StaticMemberExpression(_) => {}
        _ => report(prop.name, prop.span, offset, result),
    }
}

/// Whether the descriptor declares `required: true` (literal boolean only).
fn descriptor_required_true(descriptor: &ObjectExpression<'_>) -> bool {
    descriptor_member(descriptor, "required")
        .is_some_and(|value| matches!(value, Expression::BooleanLiteral(b) if b.value))
}

/// Whether the descriptor declares a `default` member.
fn descriptor_has_default(descriptor: &ObjectExpression<'_>) -> bool {
    descriptor_member(descriptor, "default").is_some()
}

/// Whether the descriptor's `type` is `Boolean` (a bare `Boolean`, or a `[...]`
/// array whose sole entry is `Boolean`).
fn descriptor_type_is_boolean(descriptor: &ObjectExpression<'_>) -> bool {
    let Some(type_value) = descriptor_member(descriptor, "type") else {
        return false;
    };
    expression_is_boolean(type_value)
}

/// Whether a type expression denotes the `Boolean` constructor, either directly
/// (`Boolean`) or as the only element of a one-element array (`[Boolean]`).
fn expression_is_boolean(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(identifier) => identifier.name == "Boolean",
        Expression::ArrayExpression(array) => {
            array.elements.len() == 1
                && array.elements.iter().all(|element| {
                    matches!(
                        element.as_expression(),
                        Some(Expression::Identifier(identifier)) if identifier.name == "Boolean"
                    )
                })
        }
        _ => false,
    }
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

fn report(name: &str, span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let mut message = CompactString::with_capacity(name.len() + 40);
    message.push_str("Prop '");
    message.push_str(name);
    message.push_str("' requires a default value.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("optional prop without a default", start, end)
        .with_help(
            "Add a `default` to this prop (e.g. `{ type: String, default: '' }`), or mark it \
             `required: true`. Boolean props are exempt because they default to false.",
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
