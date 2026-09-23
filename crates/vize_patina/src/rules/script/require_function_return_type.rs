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

#![allow(clippy::disallowed_macros)]

use memchr::memmem;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use vize_s0::ToCompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-function-return-type",
    description: "Require return type annotations on functions",
    default_severity: Severity::Warning,
};

/// Require function return type annotations
pub struct RequireFunctionReturnType;

impl RequireFunctionReturnType {
    /// Check if the function signature has a return type
    fn has_return_type(signature_end: &str) -> bool {
        let trimmed = signature_end.trim_start();
        // After closing paren, should have : for return type
        trimmed.starts_with(':')
    }
}

impl ScriptRule for RequireFunctionReturnType {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let bytes = source.as_bytes();

        // Fast bailout: check if there are any function definitions
        if memmem::find(bytes, b"function").is_none() && memmem::find(bytes, b"=>").is_none() {
            return;
        }

        // Find function declarations: function name(...)
        let func_finder = memmem::Finder::new(b"function ");
        let mut search_start = 0;

        while let Some(pos) = bytes
            .get(search_start..)
            .and_then(|rest| func_finder.find(rest))
        {
            let abs_pos = search_start + pos;
            search_start = abs_pos + 9;

            // Find the closing parenthesis of parameters
            let rest = source.get(abs_pos..).unwrap_or_default();
            if let Some(paren_start) = rest.find('(') {
                // Find matching close paren
                let mut depth = 0;
                let mut close_pos = None;
                for (i, c) in rest.get(paren_start..).unwrap_or_default().char_indices() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                close_pos = Some(paren_start + i);
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(cp) = close_pos {
                    let after_paren = rest.get(cp + 1..).unwrap_or_default();
                    if !Self::has_return_type(after_paren) {
                        // Extract function name for better error message
                        // (after "function " until "(")
                        let name_part = rest.get(9..paren_start).unwrap_or_default();
                        let func_name = name_part.trim();
                        let message = if func_name.is_empty() {
                            "Function is missing a return type annotation".to_compact_string()
                        } else {
                            format!(
                                "Function '{}' is missing a return type annotation",
                                func_name
                            )
                            .into()
                        };
                        result.add_diagnostic(
                            LintDiagnostic::warn(
                                META.name,
                                message,
                                (offset + abs_pos) as u32,
                                (offset + abs_pos + cp + 1) as u32,
                            )
                            .with_help("Add a return type annotation: `function fn(...): ReturnType { ... }`"),
                        );
                    }
                }
            }
        }

        // Find arrow functions: const name = (...) =>
        let arrow_finder = memmem::Finder::new(b") =>");
        search_start = 0;

        while let Some(pos) = bytes
            .get(search_start..)
            .and_then(|rest| arrow_finder.find(rest))
        {
            let abs_pos = search_start + pos;
            search_start = abs_pos + 4;

            // Check if there's a return type before ) =>
            // Look for ): pattern before the closing paren
            let before = source.get(..abs_pos).unwrap_or_default();

            // Find the matching opening paren
            let mut depth = 0;
            let mut open_pos = None;
            for (i, c) in before.char_indices().rev() {
                match c {
                    ')' => depth += 1,
                    '(' => {
                        if depth == 0 {
                            open_pos = Some(i);
                            break;
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
            }

            if let Some(op) = open_pos {
                // Check what's between open paren and closing ) =>
                let params_section = source.get(op..abs_pos + 1).unwrap_or_default();

                // A return type would look like ): Type, so after the last )
                // we should see : before =>
                if !params_section.contains("):") {
                    // Skip if this looks like a callback (inside another function call)
                    let before_paren = source.get(..op).unwrap_or_default();
                    let is_callback = before_paren
                        .trim_end()
                        .chars()
                        .last()
                        .map(|c| c == ',' || c == '(')
                        .unwrap_or(false);

                    if !is_callback {
                        result.add_diagnostic(
                            LintDiagnostic::warn(
                                META.name,
                                "Arrow function is missing a return type annotation",
                                (offset + op) as u32,
                                (offset + abs_pos + 4) as u32,
                            )
                            .with_help("Add a return type annotation: `const fn = (...): ReturnType => { ... }`"),
                        );
                    }
                }
            }
        }
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
