//! script/no-deprecated-model-definition
//!
//! Vue 3 replaced the `model` option with the `modelValue` prop and the
//! `update:modelValue` event.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   model: {
//!     prop: 'value',
//!     event: 'input'
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: ['modelValue'],
//!   emits: ['update:modelValue']
//! }
//! ```

use super::component_options::{component_definitions, property_key_name};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, ObjectPropertyKind, Program};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-model-definition",
    description: "Disallow the Vue 2 model option, which was removed in Vue 3",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "The `model` option was removed in Vue 3.";

pub struct NoDeprecatedModelDefinition;

impl ScriptRule for NoDeprecatedModelDefinition {
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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !source.contains("model") {
            return;
        }
        for options in component_definitions(program) {
            for property in &options.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed || property_key_name(&property.key) != Some("model") {
                    continue;
                }
                if matches!(
                    property.value.get_inner_expression(),
                    Expression::ObjectExpression(_)
                ) {
                    report(property.span, offset, result);
                }
            }
        }
    }
}

fn report(span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedModelDefinition;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeprecatedModelDefinition));
        linter.lint(source, 0)
    }

    #[test]
    fn flags_model_option_objects_only() {
        assert_eq!(
            lint("export default { model: { prop: 'value', event: 'input' } }").error_count,
            1
        );
        assert_eq!(
            lint("defineOptions({ model: { prop: 'value', event: 'input' } })").error_count,
            1
        );
        assert_eq!(lint("export default { model: modelOption }").error_count, 0);
    }
}
