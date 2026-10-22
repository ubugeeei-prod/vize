//! script/require-render-return
//!
//! A component `render` function must return a VNode. An empty block renders
//! nothing and is almost always a missing `return`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   render() {
//!     if (this.ready) {
//!       this.count
//!     }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   render() {
//!     return this.count
//!   }
//! }
//! ```

use super::component_options::{component_definitions, property_key_name};
use super::return_in_computed_property::body_returns_value;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, ObjectPropertyKind, Program};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-render-return",
    description: "Require the component render function to return a value",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "Expected to return a value in render function.";

pub struct RequireRenderReturn;

impl ScriptRule for RequireRenderReturn {
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
        if !source.contains("render") {
            return;
        }
        for options in component_definitions(program) {
            for property in &options.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed || property_key_name(&property.key) != Some("render") {
                    continue;
                }
                if render_returns(&property.value) == Some(false) {
                    report(property.span, offset, result);
                }
            }
        }
    }
}

/// `Some(true)` when the render function returns a value, `Some(false)` when
/// its body does not, and `None` when the value is not an inline function.
fn render_returns(value: &Expression<'_>) -> Option<bool> {
    match value.get_inner_expression() {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => Some(true),
        Expression::ArrowFunctionExpression(arrow) => {
            Some(body_returns_value(&arrow.body.statements))
        }
        Expression::FunctionExpression(function) => match function.body.as_ref() {
            Some(body) => Some(body_returns_value(&body.statements)),
            None => None,
        },
        _ => None,
    }
}

fn report(span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
}

#[cfg(test)]
mod tests {
    use super::RequireRenderReturn;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireRenderReturn));
        linter.lint(source, 0)
    }

    #[test]
    fn requires_a_returned_value() {
        assert_eq!(
            lint("export default { render() { if (this.ready) { this.count } } }").error_count,
            1
        );
        assert_eq!(
            lint("export default { render() { return this.count } }").error_count,
            0
        );
        assert_eq!(
            lint("export default { render: () => this.count }").error_count,
            0
        );
        assert_eq!(lint("export default { render: h }").error_count, 0);
    }
}
