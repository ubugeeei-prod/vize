//! script/no-with-defaults
//!
//! Discourage use of withDefaults in favor of destructuring defaults.
//!
//! Since Vue 3.5, you can use JavaScript's native destructuring with default
//! values directly in defineProps. This is more concise and idiomatic.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Using withDefaults (verbose)
//! const props = withDefaults(defineProps<{
//!   count?: number
//!   name?: string
//! }>(), {
//!   count: 0,
//!   name: 'default'
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! // Using destructuring defaults (Vue 3.5+)
//! const { count = 0, name = 'default' } = defineProps<{
//!   count?: number
//!   name?: string
//! }>()
//!
//! // Or without destructuring if defaults not needed
//! const props = defineProps<{ count: number }>()
//! ```

use oxc_ast::ast::{Expression, Program, Statement};
use oxc_span::Span;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-with-defaults",
    description: "Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)",
    default_severity: Severity::Warning,
};

/// Discourage withDefaults usage
pub struct NoWithDefaults;

impl ScriptRule for NoWithDefaults {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if !sfc.is_sfc || !sfc.is_script_setup {
            return;
        }
        // Vue processes macro statements and declarator initializers at the
        // setup block's top level. Nested function calls are ordinary code.
        for statement in &program.body {
            match statement {
                Statement::ExpressionStatement(statement) => {
                    report_macro(&statement.expression, offset, result);
                }
                Statement::VariableDeclaration(declaration) if !declaration.declare => {
                    for declarator in &declaration.declarations {
                        if let Some(initializer) = &declarator.init {
                            report_macro(initializer, offset, result);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn report_macro(expression: &Expression<'_>, offset: usize, result: &mut ScriptLintResult) {
    let Some(span) = macro_span(expression) else {
        return;
    };
    result.add_diagnostic(
        LintDiagnostic::warn(
            META.name,
            "Prefer destructuring defaults over withDefaults (Vue 3.5+)",
            offset as u32 + span.start,
            offset as u32 + span.end,
        )
        .with_help(
            "Use destructuring with defaults: \
             `const { count = 0, name = 'default' } = defineProps<Props>()`",
        ),
    );
}

fn macro_span(expression: &Expression<'_>) -> Option<Span> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    let Expression::Identifier(callee) = call.callee.without_parentheses() else {
        return None;
    };
    if call.optional || callee.name != "withDefaults" {
        return None;
    }
    let first = call.arguments.first()?.as_expression()?;
    let Expression::CallExpression(props) = first.without_parentheses() else {
        return None;
    };
    let Expression::Identifier(props_callee) = props.callee.without_parentheses() else {
        return None;
    };
    (!props.optional && props_callee.name == "defineProps").then_some(callee.span)
}

#[cfg(test)]
mod tests {
    use super::{NoWithDefaults, ScriptLintResult, ScriptRule, SfcScriptContext};
    use crate::rules::script::{ScriptLinter, script_source_type};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_l0::String;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoWithDefaults));
        linter
    }

    fn lint_setup(source: &str) -> ScriptLintResult {
        let mut sfc_source = String::from("<script setup lang=\"ts\">");
        sfc_source.push_str(source);
        sfc_source.push_str("</script>");
        let descriptor = parse_sfc(&sfc_source, SfcParseOptions::default()).unwrap();
        let setup = descriptor.script_setup.as_ref().unwrap();
        assert_eq!(setup.content, source);
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, setup.content, script_source_type()).parse();
        assert!(!parsed.panicked && parsed.diagnostics.is_empty());
        let mut result = ScriptLintResult::default();
        NoWithDefaults.check_program_with_sfc(
            &parsed.program,
            setup.content,
            // Preserve this existing unit oracle's script-local byte frame.
            0,
            SfcScriptContext {
                is_sfc: true,
                is_script_setup: descriptor.script_setup.is_some(),
                ..SfcScriptContext::default()
            },
            &mut result,
        );
        result
    }

    #[test]
    fn test_valid_destructuring_defaults() {
        let linter = create_linter();
        let result = linter.lint("const { count = 0 } = defineProps<{ count?: number }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_with_defaults() {
        let result = lint_setup(
            "const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 })",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_no_with_defaults() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<{ name: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }
}
