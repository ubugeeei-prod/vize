//! The TS lane's one type-erasure, [`erase_typescript`], over the shipped
//! `strip_typescript_from_expression` (ported verbatim).
//!
//! The shipped stripper does not type-erase by hand: it wraps the
//! expression in `const _expr_ = (…);`, parses it as TypeScript, runs the
//! oxc transformer, prints the result, and slices the initializer back
//! out. Every failure along the way (a parse diagnostic, a semantic
//! diagnostic, a transform diagnostic, a print that does not carry the
//! wrapper) returns the *original* text. Its gate is a text scan, so the
//! erasure decides what the scan missed by the parse instead.

mod detect;

use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use vize_l0::{Allocator, String, expression_guard::expression_is_safe_to_parse};

pub(in crate::emit) use detect::needs_typescript_stripping;

/// The JavaScript spelling of authored template text. `proven_js`: a
/// retained AST passed the JS-module dialect gate for these bytes.
pub(in crate::emit) fn erase_typescript(content: &str, proven_js: bool) -> String {
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
pub(in crate::emit) fn erase_typescript_params(params: &str) -> Option<String> {
    if !expression_is_safe_to_parse(params) {
        return None;
    }
    let mut arrow = String::with_capacity(params.len() + 12);
    arrow.push('(');
    arrow.push_str(params);
    arrow.push_str("\n) => null");
    let js = Allocator::new();
    if Parser::new(js.as_oxc(), arrow.as_str(), js_module())
        .parse_expression()
        .is_ok()
    {
        return None;
    }
    let mut wrapped = String::from("const _p_ = ");
    wrapped.push_str(arrow.as_str());
    wrapped.push(';');
    transform_print(wrapped.as_str(), |printed| {
        let rest = printed.strip_prefix("const _p_ = (")?;
        let (inner, _) = rest.rsplit_once(") => null;")?;
        Some(String::from(inner))
    })
}

/// The shipped stripper: gated on the detection scan, identity on any
/// refusal.
pub(in crate::emit) fn strip_typescript_from_expression(content: &str) -> String {
    // Inputs that would overflow the parser stack pass through unchanged
    // so the surrounding lane emits its normal diagnostic (#956).
    if !expression_is_safe_to_parse(content) {
        return String::from(content);
    }
    if !needs_typescript_stripping(content) {
        return String::from(content);
    }
    let mut wrapped = String::with_capacity(content.len() + 18);
    wrapped.push_str("const _expr_ = (");
    wrapped.push_str(content);
    wrapped.push_str(");");
    transform_print(wrapped.as_str(), extract_initializer).unwrap_or_else(|| String::from(content))
}

fn js_module() -> SourceType {
    SourceType::default().with_module(true)
}

/// The legacy chain's two JS parses: the wrapped expression, then the
/// program. Unparseable depth is not this lane's to decide.
fn is_javascript(content: &str) -> bool {
    if !expression_is_safe_to_parse(content) {
        return true;
    }
    let mut wrapped = String::with_capacity(content.len() + 3);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push_str("\n)");
    let expression = Allocator::new();
    if Parser::new(expression.as_oxc(), wrapped.as_str(), js_module())
        .parse_expression()
        .is_ok()
    {
        return true;
    }
    let program = Allocator::new();
    Parser::new(program.as_oxc(), content, js_module())
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
    transform_print(wrapped.as_str(), extract_initializer)
}

/// TypeScript the scan missed, as a statement list (a handler body).
fn reprint_program(content: &str) -> Option<String> {
    transform_print(content, |printed| Some(String::from(printed.trim_end())))
}

/// Parse as TypeScript, erase, print, and `finish` the printed text;
/// `None` on any diagnostic.
fn transform_print<T>(source: &str, finish: impl FnOnce(&str) -> Option<T>) -> Option<T> {
    let allocator = Allocator::new();
    let parse_result = Parser::new(allocator.as_oxc(), source, SourceType::ts()).parse();
    if !parse_result.diagnostics.is_empty() {
        return None;
    }
    let mut program = parse_result.program;
    // `with_enum_eval`: the enum transform panics without evaluated members.
    let semantic = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(&program);
    if !semantic.diagnostics.is_empty() {
        return None;
    }
    let scoping = semantic.semantic.into_scoping();
    let options = TransformOptions::default();
    let transformed = Transformer::new(allocator.as_oxc(), std::path::Path::new(""), &options)
        .build_with_scoping(scoping, &mut program);
    if !transformed.diagnostics.is_empty() {
        return None;
    }
    finish(Codegen::new().build(&program).code.as_str())
}

/// The printed program is `const _expr_ = …;`, with the parentheses kept
/// or dropped by the printer; slice the initializer back out.
fn extract_initializer(printed: &str) -> Option<String> {
    const PREFIX: &str = "const _expr_ = ";
    let (_, initializer) = printed.split_once(PREFIX)?;
    let (initializer, _) = initializer.rsplit_once(';')?;
    let expr = initializer.trim();
    if has_matching_outer_parens(expr)
        && let Some(inner) = expr.strip_prefix('(').and_then(|e| e.strip_suffix(')'))
    {
        return Some(String::from(inner));
    }
    Some(String::from(expr))
}

/// Whether the outermost parentheses pair with each other: `(foo)` does,
/// `(isOpen) => foo(x)` does not.
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
mod tests {
    use super::{erase_typescript, erase_typescript_params, strip_typescript_from_expression};

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
        assert_eq!(erase_typescript("foo as ", false).as_str(), "foo as ");
    }

    #[test]
    fn erasure_decides_what_the_scan_misses_by_the_parse() {
        let erased = |text: &str| erase_typescript(text, false);
        assert_eq!(erased("(e): void => pick(e)").as_str(), "(e) => pick(e)");
        assert_eq!(erased("pick<Item>").as_str(), "pick");
        assert_eq!(erased("<any>count").as_str(), "count");
        assert_eq!(
            erased("((x?: number) => x)(count)").as_str(),
            "((x) => x)(count)"
        );
        assert_eq!(
            erased("function (this: Window, x: number): number { return x }").as_str(),
            "function(x) {\n\treturn x;\n}"
        );
        assert_eq!(
            erased("const n: number = count; pick(n)").as_str(),
            "const n = count;\npick(n);"
        );
        // JavaScript keeps its bytes: no reprint without TypeScript.
        assert_eq!(
            erased("function (x) { return x }").as_str(),
            "function (x) { return x }"
        );
        assert_eq!(erased("a++; b++").as_str(), "a++; b++");
        // A proven-JS retained AST skips the parse entirely.
        assert_eq!(erase_typescript("pick<Item>", true).as_str(), "pick<Item>");
    }

    #[test]
    fn parameter_lists_erase_their_annotations() {
        assert_eq!(
            erase_typescript_params("{ row }: { row: Row }").as_deref(),
            Some("{ row }")
        );
        assert_eq!(
            erase_typescript_params("props: Props").as_deref(),
            Some("props")
        );
        assert_eq!(erase_typescript_params("{ row = fallback }"), None);
        assert_eq!(erase_typescript_params("{ row: "), None);
    }
}
