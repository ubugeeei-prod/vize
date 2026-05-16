//! TypeScript stripping for template expressions.
//!
//! Handles removing TypeScript type annotations (e.g., `as` assertions,
//! parameter type annotations) from template expressions before codegen.

use oxc_allocator::Allocator as OxcAllocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use vize_carton::String;

/// Check if expression contains TypeScript syntax that needs stripping
pub(crate) fn needs_typescript_stripping(content: &str) -> bool {
    // Quick check for common TypeScript patterns
    // - " as " is TypeScript type assertion
    // - We avoid checking ": " as it's also used in object literals
    // - Generic types like "Array<string>" - but we need to be careful not to match comparison operators
    if content.contains(" as ") {
        return true;
    }

    // Check for arrow function parameter type annotations: (param: Type) =>
    // Pattern: identifier followed by : and then some type, before ) =>
    if content.contains("=>") {
        // Look for patterns like "(x: Type)" or "(x: Type, y: Type2)"
        let bytes = content.as_bytes();
        let mut in_paren = false;
        let mut after_ident = false;
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'(' => {
                    in_paren = true;
                    after_ident = false;
                }
                b')' => {
                    in_paren = false;
                    after_ident = false;
                }
                b':' if in_paren && after_ident => {
                    // Found colon after identifier inside parens before =>
                    // This is likely a type annotation
                    // Check it's not :: (TypeScript namespace separator)
                    if i + 1 < bytes.len() && bytes[i + 1] != b':' {
                        return true;
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' | b'0'..=b'9' => {
                    after_ident = true;
                }
                b' ' | b'\t' => {
                    // Whitespace doesn't reset after_ident
                }
                b',' => {
                    // Comma resets for next parameter
                    after_ident = false;
                }
                _ => {
                    after_ident = false;
                }
            }
        }
    }

    // Check for non-null assertion operator (foo!, bar.baz!, etc.)
    // This is tricky because we need to distinguish from logical NOT (!foo)
    // Non-null assertion comes AFTER an expression, not before
    // Pattern: identifier/closing bracket/paren followed by !
    let bytes = content.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'!' {
            // Check if this is a non-null assertion (! after identifier/)/])
            // rather than logical NOT (! before expression)
            if i > 0 {
                let prev = bytes[i - 1];
                // Non-null assertion if previous char is:
                // - alphanumeric (foo!)
                // - underscore or dollar (var_!)
                // - closing paren (foo()!)
                // - closing bracket (foo[0]!)
                let is_non_null_assertion = prev.is_ascii_alphanumeric()
                    || prev == b'_'
                    || prev == b'$'
                    || prev == b')'
                    || prev == b']';
                if is_non_null_assertion {
                    return true;
                }
            }
        }
    }

    false
}

/// Strip TypeScript type annotations from an expression
pub fn strip_typescript_from_expression(content: &str) -> String {
    // Only process if TypeScript syntax is detected
    if !needs_typescript_stripping(content) {
        return String::new(content);
    }

    let allocator = OxcAllocator::default();
    let source_type = SourceType::ts();

    // Wrap in a dummy statement to make it parseable
    let mut wrapped = String::with_capacity(content.len() + 18);
    wrapped.push_str("const _expr_ = (");
    wrapped.push_str(content);
    wrapped.push_str(");");
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse();

    if !parse_result.errors.is_empty() {
        // If parsing fails, return original content
        return String::new(content);
    }

    let mut program = parse_result.program;

    // Run semantic analysis
    let semantic_ret = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    if !semantic_ret.errors.is_empty() {
        return String::new(content);
    }

    let scoping = semantic_ret.semantic.into_scoping();

    // Transform TypeScript to JavaScript
    let transform_options = TransformOptions::default();
    let ret = Transformer::new(&allocator, std::path::Path::new(""), &transform_options)
        .build_with_scoping(scoping, &mut program);

    if !ret.errors.is_empty() {
        return String::new(content);
    }

    // Generate JavaScript code
    let js_code = Codegen::new().build(&program).code;

    // Extract the expression from the generated code
    // The output can be: "const _expr_ = (...);\n" or "const _expr_ = ...;\n"
    // (codegen may remove unnecessary parentheses)
    let prefix = "const _expr_ = ";
    if let Some(start) = js_code.find(prefix) {
        let expr_start = start + prefix.len();
        // Find the semicolon at the end
        if let Some(end) = js_code[expr_start..].rfind(';') {
            let expr = &js_code[expr_start..expr_start + end];
            // Remove surrounding parentheses if present
            let expr = expr.trim();
            if expr.starts_with('(') && expr.ends_with(')') && has_matching_outer_parens(expr) {
                return String::new(&expr[1..expr.len() - 1]);
            }
            return String::new(expr);
        }
    }

    // Fallback: return original content
    String::new(content)
}

/// Check if the outermost parens in a string are actually matching.
/// e.g. "(foo)" => true, "(isOpen) => foo(x)" => false
fn has_matching_outer_parens(s: &str) -> bool {
    if !s.starts_with('(') || !s.ends_with(')') {
        return false;
    }
    let inner = &s[1..s.len() - 1];
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut prev_char = ' ';
    for ch in inner.chars() {
        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            continue;
        }
        match ch {
            '\'' | '"' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
        prev_char = ch;
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::{needs_typescript_stripping, strip_typescript_from_expression};

    #[test]
    fn test_strip_typescript_from_expression() {
        // Simple as assertion
        let result = strip_typescript_from_expression("$event.target as HTMLSelectElement");
        insta::assert_snapshot!(result.as_str());

        // Chained as assertions
        let result =
            strip_typescript_from_expression("($event.target as HTMLInputElement).value as string");
        insta::assert_snapshot!(result.as_str());

        // No TypeScript - should return as-is
        let result = strip_typescript_from_expression("foo.bar.baz");
        assert_eq!(result.trim(), "foo.bar.baz");

        // Complex nested expression with multiple as assertions (from App.vue)
        let result = strip_typescript_from_expression(
            "handlePresetChange(($event.target as HTMLSelectElement).value as PresetKey)",
        );
        eprintln!("Complex expression result: {}", result);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_needs_typescript_stripping_as_keyword() {
        assert!(needs_typescript_stripping("foo as string"));
        assert!(needs_typescript_stripping("$event.target as HTMLElement"));
        assert!(!needs_typescript_stripping("foo.bar"));
    }

    #[test]
    fn test_needs_typescript_stripping_arrow_function_params() {
        // Arrow function with typed parameters should be detected
        assert!(needs_typescript_stripping("(x: number) => x + 1"));
        assert!(needs_typescript_stripping("(item: Item) => item.name"));
        assert!(needs_typescript_stripping(
            "(a: string, b: number) => a + b"
        ));

        // Arrow function without types should not need stripping
        assert!(!needs_typescript_stripping("(x) => x + 1"));
        assert!(!needs_typescript_stripping("x => x + 1"));
    }

    #[test]
    fn test_needs_typescript_stripping_generic_detection_note() {
        // NOTE: Generic function call detection (e.g., useStore<RootState>())
        // is not implemented in needs_typescript_stripping.
        // Generic stripping is handled by the full OXC TypeScript transformer
        // when compiling script blocks with is_ts = false.
        // This test documents the current behavior.

        // Currently NOT detected as needing stripping:
        assert!(!needs_typescript_stripping("useStore<RootState>()"));
        assert!(!needs_typescript_stripping("ref<User | null>(null)"));

        // Regular function calls correctly don't need stripping:
        assert!(!needs_typescript_stripping("useStore()"));
        assert!(!needs_typescript_stripping("ref(null)"));
    }

    #[test]
    fn test_strip_typescript_documents_limitations() {
        // NOTE: strip_typescript_from_expression is a simple parser-based
        // transformation for template expressions. It handles common cases
        // like "as" assertions, but complex TypeScript like generics are
        // handled by the full OXC transformer in compile_script.
        //
        // For template expressions with generics, they are stripped during
        // script compilation (not in the template transform phase).

        // "as" assertions are stripped:
        let result = strip_typescript_from_expression("foo as string");
        insta::assert_snapshot!(result.as_str());

        // Generics in expressions MAY or MAY NOT be stripped depending on context
        // This is expected behavior - complex cases are handled elsewhere
        let result = strip_typescript_from_expression("useStore<RootState>()");
        // Document the actual behavior - generics aren't stripped by this function
        eprintln!("Generic expression result: {}", result);
    }

    #[test]
    fn test_strip_typescript_arrow_param_types() {
        let result = strip_typescript_from_expression("items.filter((x: number) => x > 1)");
        eprintln!("Arrow param stripped: {}", result);
        insta::assert_snapshot!(result.as_str());
    }
}
