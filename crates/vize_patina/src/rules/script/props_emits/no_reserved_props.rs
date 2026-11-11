//! script/no-reserved-props
//!
//! Disallow reserved names in a component's props declaration.
//!
//! Vue reserves a handful of attribute names for its own runtime use, so a prop
//! declared with one of these names is shadowed by Vue's handling and never
//! receives the value the parent passed. The Vue 3 names are `key`, `ref`,
//! `ref_for`, and `ref_key`, plus any name beginning with `$` (reserved for
//! Vue's internal instance properties). Vue 3 accepts a prop named `is`.
//!
//! Covers the object form (`{ key: ... }`), the array form (`['key']`,
//! including a static template-literal element), and a same-file type argument
//! of `defineProps` / `withDefaults`. The declarations may come from the
//! Options API `props` option (including `defineComponent({...})` and a
//! same-file identifier-bound object) or from `<script setup>`.
//!
//! Port of [`vue/no-reserved-props`](https://eslint.vuejs.org/rules/no-reserved-props.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     ref: String,   // reserved
//!     $foo: Number    // `$`-prefixed names are reserved
//!   }
//! }
//!
//! export default {
//!   props: ['key']    // reserved (array form)
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     name: String,
//!     refValue: Number
//!   }
//! }
//! ```

use oxc_ast::ast::{Expression, Program};
use oxc_span::GetSpan;

use vize_l0::CompactString;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::props_source::{PropsSource, collect_runtime_props};
use super::type_prop_names::same_file_type_prop_names;
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-reserved-props",
    description: "Disallow reserved names in a component's props declaration",
    default_severity: Severity::Error,
};

/// Names Vue 3 reserves for its own use; a prop must not be declared with one.
const RESERVED_PROP_NAMES: &[&str] = &["key", "ref", "ref_for", "ref_key"];

/// Disallow reserved prop names.
pub struct NoReservedProps;

impl ScriptRule for NoReservedProps {
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
                if is_reserved(prop.name) {
                    report(prop.name, prop.key.span(), offset, result);
                }
            }
            for literal in source.array_names() {
                if is_reserved(literal.value.as_str()) {
                    report(literal.value.as_str(), literal.span, offset, result);
                }
            }
            if let PropsSource::Array(array) = source {
                for element in &array.elements {
                    let Some(expression) = element.as_expression() else {
                        continue;
                    };
                    let Expression::TemplateLiteral(template) = expression.get_inner_expression()
                    else {
                        continue;
                    };
                    let Some(name) = template.single_quasi() else {
                        continue;
                    };
                    if is_reserved(name.as_str()) {
                        report(name.as_str(), template.span(), offset, result);
                    }
                }
            }
        }
        for (name, span) in same_file_type_prop_names(program) {
            if is_reserved(name.as_str()) {
                report(name.as_str(), span, offset, result);
            }
        }
    }
}

/// Whether a prop name is reserved: one of the fixed names, or `$`-prefixed.
fn is_reserved(name: &str) -> bool {
    name.starts_with('$') || RESERVED_PROP_NAMES.contains(&name)
}

fn report(name: &str, span: oxc_span::Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let mut message = CompactString::with_capacity(name.len() + 36);
    message.push_str("Prop name '");
    message.push_str(name);
    message.push_str("' is reserved.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("reserved prop name", start, end)
        .with_help(
            "Vue reserves `key`, `ref`, `ref_for`, `ref_key`, and any `$`-prefixed name. \
             Rename this prop to something else.",
        );
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
mod tests;
