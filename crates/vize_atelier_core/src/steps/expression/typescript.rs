//! TypeScript stripping for template expressions.
//!
//! Handles removing TypeScript type annotations (e.g., `as` assertions,
//! parameter type annotations) from template expressions before codegen.

use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use vize_l0::String;

/// Check if expression contains TypeScript syntax that needs stripping
pub(crate) fn needs_typescript_stripping(content: &str) -> bool {
    // Quick check for common TypeScript patterns
    // - " as " is TypeScript type assertion
    // - We avoid checking ": " as it's also used in object literals
    // - Generic types like "Array<string>" - but we need to be careful not to match comparison operators
    if content.contains(" as ") {
        return true;
    }

    if contains_unquoted_word(content, "satisfies") {
        return true;
    }

    if contains_generic_call(content) {
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
                    if bytes.get(i + 1).is_some_and(|&next| next != b':') {
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
            if let Some(&prev) = i.checked_sub(1).and_then(|p| bytes.get(p)) {
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

fn contains_unquoted_word(content: &str, word: &str) -> bool {
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in content.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        let (head, tail) = content.split_at_checked(index).unwrap_or_default();
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            _ if tail.starts_with(word) => {
                let before = head.chars().next_back();
                let after = tail.get(word.len()..).and_then(|rest| rest.chars().next());
                let before_boundary = before.is_none_or(|ch| !is_ident_char(ch));
                let after_boundary = after.is_none_or(|ch| !is_ident_char(ch));
                if before_boundary && after_boundary {
                    return true;
                }
            }
            _ => {}
        }
        prev = ch;
    }

    false
}

fn contains_generic_call(content: &str) -> bool {
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in content.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '<' if previous_non_whitespace(content, index).is_some_and(is_ident_char) => {
                if let Some(close) = find_matching_angle(content, index) {
                    let after = content.get(close + 1..).unwrap_or_default().trim_start();
                    if after.starts_with('(') {
                        return true;
                    }
                }
            }
            _ => {}
        }
        prev = ch;
    }

    false
}

fn previous_non_whitespace(content: &str, index: usize) -> Option<char> {
    let head = content.get(..index)?;
    head.chars().rev().find(|ch| !ch.is_whitespace())
}

fn find_matching_angle(content: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (relative, ch) in content.get(open_index..).unwrap_or_default().char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_index + relative);
                }
            }
            _ => {}
        }
        prev = ch;
    }

    None
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

/// The JavaScript spelling of authored template text under `is_ts`, the TS
/// lane's one erasure. The stripper below is gated on a text scan; what the
/// scan misses is decided by the parse. `proven_js`: a retained AST passed
/// the JS-module dialect gate for these bytes.
pub fn erase_typescript(content: &str, proven_js: bool) -> String {
    let stripped = strip_typescript_from_expression(content);
    if proven_js || stripped.as_str() != content || is_javascript(content) {
        return stripped;
    }
    reprint_expression(content)
        .or_else(|| reprint_program(content))
        .unwrap_or(stripped)
}

/// A parameter list (slot props, a `v-for` alias) as JavaScript; `None`
/// when it already is, or when it does not parse as TypeScript either.
pub fn erase_typescript_params(params: &str) -> Option<String> {
    if !super::expression_is_safe_to_parse(params) {
        return None;
    }
    let mut arrow = String::with_capacity(params.len() + 12);
    arrow.push('(');
    arrow.push_str(params);
    arrow.push_str("\n) => null");
    let allocator = crate::expr_parse_probe::parse_arena();
    if Parser::new(&allocator, &arrow, SourceType::default().with_module(true))
        .parse_expression()
        .is_ok()
    {
        return None;
    }
    let mut wrapped = String::new("const _p_ = ");
    wrapped.push_str(&arrow);
    wrapped.push(';');
    transform_print(&wrapped, |printed| {
        let rest = printed.strip_prefix("const _p_ = (")?;
        let (inner, _) = rest.rsplit_once(") => null;")?;
        Some(String::new(inner))
    })
}

/// Strip TypeScript type annotations from an expression
pub fn strip_typescript_from_expression(content: &str) -> String {
    // Skip parsing for inputs that would overflow the parser stack — return
    // the original content unchanged so the surrounding lane emits a
    // normal diagnostic instead of aborting. (#956)
    if !super::expression_is_safe_to_parse(content) {
        return String::new(content);
    }
    // Only process if TypeScript syntax is detected
    if !needs_typescript_stripping(content) {
        return String::new(content);
    }

    // Wrap in a dummy statement to make it parseable
    let mut wrapped = String::with_capacity(content.len() + 18);
    wrapped.push_str("const _expr_ = (");
    wrapped.push_str(content);
    wrapped.push_str(");");
    // Fallback: return original content
    transform_print(&wrapped, extract_initializer).unwrap_or_else(|| String::new(content))
}

/// The legacy chain's two JS parses: the wrapped expression, then the
/// program. Unparseable depth is not this lane's to decide.
fn is_javascript(content: &str) -> bool {
    if !super::expression_is_safe_to_parse(content) {
        return true;
    }
    let source_type = SourceType::default().with_module(true);
    let mut wrapped = String::with_capacity(content.len() + 3);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push_str("\n)");
    let expression = crate::expr_parse_probe::parse_arena();
    if Parser::new(&expression, &wrapped, source_type)
        .parse_expression()
        .is_ok()
    {
        return true;
    }
    let program = crate::expr_parse_probe::parse_arena();
    Parser::new(&program, content, source_type)
        .parse()
        .diagnostics
        .is_empty()
}

/// TypeScript the scan missed, as one expression.
fn reprint_expression(content: &str) -> Option<String> {
    let mut wrapped = String::with_capacity(content.len() + 19);
    wrapped.push_str("const _expr_ = (");
    wrapped.push_str(content);
    wrapped.push_str("\n);");
    transform_print(&wrapped, extract_initializer)
}

/// TypeScript the scan missed, as a statement list (a handler body).
fn reprint_program(content: &str) -> Option<String> {
    transform_print(content, |printed| Some(String::new(printed.trim_end())))
}

/// Parse as TypeScript, erase, print, and `finish` the printed text;
/// `None` on any diagnostic.
fn transform_print<T>(source: &str, finish: impl FnOnce(&str) -> Option<T>) -> Option<T> {
    let allocator = crate::expr_parse_probe::parse_arena();
    let parse_result = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !parse_result.diagnostics.is_empty() {
        return None;
    }

    let mut program = parse_result.program;

    // `with_enum_eval`: OXC 0.142's enum transform panics without evaluated members.
    let semantic_ret = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(&program);

    if !semantic_ret.diagnostics.is_empty() {
        return None;
    }

    let scoping = semantic_ret.semantic.into_scoping();

    // Transform TypeScript to JavaScript
    let transform_options = TransformOptions::default();
    let ret = Transformer::new(&allocator, std::path::Path::new(""), &transform_options)
        .build_with_scoping(scoping, &mut program);

    if !ret.diagnostics.is_empty() {
        return None;
    }

    finish(&Codegen::new().build(&program).code)
}

/// The printed program is `const _expr_ = …;`, with the parentheses kept
/// or dropped by the printer; slice the initializer back out.
fn extract_initializer(js_code: &str) -> Option<String> {
    // The output can be: "const _expr_ = (...);\n" or "const _expr_ = ...;\n"
    // (codegen may remove unnecessary parentheses)
    let (_, rest) = js_code.split_once("const _expr_ = ")?;
    let (expr, _) = rest.rsplit_once(';')?;
    // Remove surrounding parentheses if present
    let expr = expr.trim();
    if has_matching_outer_parens(expr) {
        return Some(String::new(expr.get(1..expr.len() - 1).unwrap_or_default()));
    }
    Some(String::new(expr))
}

/// Check if the outermost parens in a string are actually matching.
/// e.g. "(foo)" => true, "(isOpen) => foo(x)" => false
fn has_matching_outer_parens(s: &str) -> bool {
    let Some(inner) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) else {
        return false;
    };
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
#[expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
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
    fn test_needs_typescript_stripping_generic_calls() {
        assert!(needs_typescript_stripping("useStore<RootState>()"));
        assert!(needs_typescript_stripping("ref<User | null>(null)"));
        assert!(needs_typescript_stripping(
            "factory<Map<string, number>>(value)"
        ));

        // Regular function calls correctly don't need stripping:
        assert!(!needs_typescript_stripping("useStore()"));
        assert!(!needs_typescript_stripping("ref(null)"));
    }

    #[test]
    fn test_needs_typescript_stripping_satisfies_operator() {
        assert!(needs_typescript_stripping("payload satisfies Payload"));
        assert!(!needs_typescript_stripping(
            "'payload satisfies Payload'.includes(query)"
        ));
    }

    #[test]
    fn test_strip_typescript_generic_calls_and_satisfies() {
        let result = strip_typescript_from_expression("foo as string");
        assert_eq!(result.as_str(), "foo");

        let result = strip_typescript_from_expression("useStore<RootState>()");
        assert_eq!(result.as_str(), "useStore()");

        let result = strip_typescript_from_expression("ref<User | null>(null)");
        assert_eq!(result.as_str(), "ref(null)");

        let result = strip_typescript_from_expression("payload satisfies Payload");
        assert_eq!(result.as_str(), "payload");
    }

    #[test]
    fn test_strip_typescript_arrow_param_types() {
        let result = strip_typescript_from_expression("items.filter((x: number) => x > 1)");
        eprintln!("Arrow param stripped: {}", result);
        insta::assert_snapshot!(result.as_str());
    }
}
