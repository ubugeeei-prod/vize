//! `strip_typescript_from_expression`, ported verbatim.
//!
//! The shipped TS lane does not type-erase by hand: it wraps the
//! expression in `const _expr_ = (…);`, parses it as TypeScript, runs the
//! oxc transformer, prints the result, and slices the initializer back
//! out. Every failure along the way (a parse diagnostic, a semantic
//! diagnostic, a transform diagnostic, a print that does not carry the
//! wrapper) returns the *original* text, so the emitter must reproduce
//! those returns as faithfully as the successful strip.

mod detect;

use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use vize_s0::{Allocator, String, expression_guard::expression_is_safe_to_parse};

pub(in crate::emit) use detect::needs_typescript_stripping;

/// The TS-stripped spelling of `content`, or `content` itself when the
/// scan sees no TypeScript or any stage of the round-trip refuses.
pub(in crate::emit) fn strip_typescript_from_expression(content: &str) -> String {
    // Inputs that would overflow the parser stack pass through unchanged
    // so the surrounding lane emits its normal diagnostic (#956).
    if !expression_is_safe_to_parse(content) {
        return String::from(content);
    }
    if !needs_typescript_stripping(content) {
        return String::from(content);
    }

    let allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 18);
    wrapped.push_str("const _expr_ = (");
    wrapped.push_str(content);
    wrapped.push_str(");");
    let parse_result = Parser::new(allocator.as_oxc(), wrapped.as_str(), SourceType::ts()).parse();
    if !parse_result.diagnostics.is_empty() {
        return String::from(content);
    }

    let mut program = parse_result.program;
    // `with_enum_eval`: the enum transform panics without evaluated members.
    let semantic = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(&program);
    if !semantic.diagnostics.is_empty() {
        return String::from(content);
    }
    let scoping = semantic.semantic.into_scoping();

    let options = TransformOptions::default();
    let transformed = Transformer::new(allocator.as_oxc(), std::path::Path::new(""), &options)
        .build_with_scoping(scoping, &mut program);
    if !transformed.diagnostics.is_empty() {
        return String::from(content);
    }

    let printed = Codegen::new().build(&program).code;
    extract_initializer(printed.as_str()).unwrap_or_else(|| String::from(content))
}

/// The printed program is `const _expr_ = …;`, with the parentheses kept
/// or dropped by the printer; slice the initializer back out.
fn extract_initializer(printed: &str) -> Option<String> {
    const PREFIX: &str = "const _expr_ = ";
    let start = printed.find(PREFIX)? + PREFIX.len();
    let end = printed[start..].rfind(';')?;
    let expr = printed[start..start + end].trim();
    if expr.starts_with('(') && expr.ends_with(')') && has_matching_outer_parens(expr) {
        return Some(String::from(&expr[1..expr.len() - 1]));
    }
    Some(String::from(expr))
}

/// Whether the outermost parentheses pair with each other: `(foo)` does,
/// `(isOpen) => foo(x)` does not.
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
    use super::strip_typescript_from_expression;

    #[test]
    fn strips_the_shipped_typescript_shapes() {
        assert_eq!(
            strip_typescript_from_expression("foo as string").as_str(),
            "foo"
        );
        assert_eq!(
            strip_typescript_from_expression("useStore<RootState>()").as_str(),
            "useStore()"
        );
        assert_eq!(
            strip_typescript_from_expression("ref<User | null>(null)").as_str(),
            "ref(null)"
        );
        assert_eq!(
            strip_typescript_from_expression("payload satisfies Payload").as_str(),
            "payload"
        );
        assert_eq!(
            strip_typescript_from_expression("foo.bar.baz").as_str(),
            "foo.bar.baz"
        );
    }

    #[test]
    fn unparseable_typescript_passes_through() {
        assert_eq!(
            strip_typescript_from_expression("foo as ").as_str(),
            "foo as "
        );
    }
}
