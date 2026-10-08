//! script/require-function-return-type
//!
//! Require return type annotations on functions.
//!
//! Function definitions should declare their return type explicitly
//! to improve code readability and catch type errors early.
//! This applies to named functions, arrow functions assigned to const,
//! and methods in objects.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const add = (a: number, b: number) => {
//!   return a + b
//! }
//!
//! function greet(name: string) {
//!   return `Hello, ${name}`
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! const add = (a: number, b: number): number => {
//!   return a + b
//! }
//!
//! function greet(name: string): string {
//!   return `Hello, ${name}`
//! }
//! ```
//!
//! ### Exceptions
//! - Callback functions passed as arguments (inferred from context)
//! - Arrow functions without block body (e.g., `x => x + 1`)

use oxc_ast::ast::{ArrowFunctionExpression, Function, Program};
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function},
};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::cstr;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-function-return-type",
    description: "Require return type annotations on functions",
    default_severity: Severity::Warning,
};

/// Require function return type annotations.
pub struct RequireFunctionReturnType;

impl ScriptRule for RequireFunctionReturnType {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

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
        ReturnTypeVisitor {
            source,
            offset,
            result,
        }
        .visit_program(program);
    }
}

struct ReturnTypeVisitor<'source, 'result> {
    source: &'source str,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for ReturnTypeVisitor<'_, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if function.body.is_some() && function.return_type.is_none() {
            // Keep the established function-keyword span and message. Method
            // signatures and ambient declarations are not runtime functions.
            let header = self
                .source
                .get(function.span.start as usize..function.params.span.start as usize)
                .unwrap_or_default();
            if let Some(keyword) = header.find("function ") {
                let name = header.get(keyword + 9..).unwrap_or_default().trim();
                let message = if name.is_empty() {
                    "Function is missing a return type annotation".into()
                } else {
                    cstr!("Function '{}' is missing a return type annotation", name)
                };
                self.result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        message,
                        self.offset as u32 + function.span.start + keyword as u32,
                        self.offset as u32 + function.params.span.end,
                    )
                    .with_help(
                        "Add a return type annotation: `function fn(...): ReturnType { ... }`",
                    ),
                );
            }
        }
        walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let start = arrow.params.span.start as usize;
        let end = arrow.params.span.end as usize;
        let params = self.source.get(start..end).unwrap_or_default();
        let before = self.source.get(..start).unwrap_or_default();
        // Retain the current parenthesized-arrow and callback policy while
        // selecting real ArrowFunctionExpression nodes, never TSFunctionType.
        let callback = matches!(before.trim_end().chars().last(), Some(',' | '('));
        if arrow.return_type.is_none() && params.starts_with('(') && !callback {
            let tail = self
                .source
                .get(end..arrow.body.span.start as usize)
                .unwrap_or_default();
            if let Some(operator) = tail.find("=>") {
                self.result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Arrow function is missing a return type annotation",
                        (self.offset + start) as u32,
                        (self.offset + end + operator + 2) as u32,
                    )
                    .with_help(
                        "Add a return type annotation: `const fn = (...): ReturnType => { ... }`",
                    ),
                );
            }
        }
        walk_arrow_function_expression(self, arrow);
    }
}

#[cfg(test)]
mod tests {
    use super::RequireFunctionReturnType;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireFunctionReturnType));
        linter
    }

    #[test]
    fn test_valid_function_with_return_type() {
        let linter = create_linter();
        let result = linter.lint(
            "function greet(name: string): string { return `Hello, ${name}` }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_function_without_return_type() {
        let linter = create_linter();
        let result = linter.lint(
            "function greet(name: string) { return `Hello, ${name}` }",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_no_functions() {
        let linter = create_linter();
        let result = linter.lint("const x = 1\nconst y = 2", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_arrow_span_is_a_byte_offset_with_non_ascii_params() {
        let linter = create_linter();
        let source = "const 日=(v = '日') => v";
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 1);
        let open = source.find('(').unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, open);
    }
}
