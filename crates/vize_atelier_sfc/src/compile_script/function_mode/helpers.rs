//! Utility functions for function-mode script compilation.
//!
//! Contains helpers for backtick counting, TypeScript type alias detection,
//! JavaScript reserved word checking, and top-level await detection.

use oxc_allocator::Allocator;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_for_of_statement, walk_function};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{FxHashSet, String, ToCompactString};

use crate::compile_script::typescript::transform_typescript_to_js;

/// Count unescaped backticks in a line, ignoring those inside regular strings.
/// Returns the change in template literal depth (positive = more opens, negative = more closes).
/// Since backticks toggle depth, we return the count which should be added to track depth.
pub(crate) fn count_unescaped_backticks(line: &str) -> i32 {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < chars.len() {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };

        // Track regular string state (but don't track template literals here,
        // we're counting backticks to determine template literal depth)
        if c == '\'' && prev != Some('\\') && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && prev != Some('\\') && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if c == '`' && prev != Some('\\') && !in_single_quote && !in_double_quote {
            // Found an unescaped backtick outside of regular strings
            count += 1;
        }

        i += 1;
    }

    count
}

/// Check if a line starts a TypeScript type alias declaration.
pub(crate) fn is_typescript_type_alias(line: &str) -> bool {
    let trimmed = line.trim_start();
    let prefix = if trimmed.starts_with("export type ") {
        "export type "
    } else if trimmed.starts_with("type ") {
        "type "
    } else {
        return false;
    };

    let rest = trimmed[prefix.len()..].trim_start();
    let mut chars = rest.chars();
    matches!(
        chars.next(),
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$'
    )
}

/// Check if an identifier is a JavaScript reserved word (avoid shorthand).
pub(crate) fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}

/// Collect runtime identifier references from setup code after stripping TypeScript syntax.
pub(crate) fn collect_runtime_identifier_references(code: &str) -> FxHashSet<String> {
    let runtime_code = transform_typescript_to_js(code);

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &runtime_code, SourceType::default());
    let parse_result = parser.parse();

    if !parse_result.errors.is_empty() {
        return FxHashSet::default();
    }

    #[derive(Default)]
    struct RuntimeIdentifierVisitor {
        identifiers: FxHashSet<String>,
    }

    impl<'a> Visit<'a> for RuntimeIdentifierVisitor {
        fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
            self.identifiers
                .insert(ident.name.as_str().to_compact_string());
        }
    }

    let mut visitor = RuntimeIdentifierVisitor::default();
    visitor.visit_program(&parse_result.program);
    visitor.identifiers
}

/// Detect top-level await in setup code (ignores awaits inside nested functions).
pub fn contains_top_level_await(code: &str, is_ts: bool) -> bool {
    let allocator = Allocator::default();
    let source_type = if is_ts {
        SourceType::ts()
    } else {
        SourceType::default()
    };

    let mut wrapped = String::with_capacity(code.len() + 28);
    wrapped.push_str("async function __temp__() {\n");
    wrapped.push_str(code);
    wrapped.push_str("\n}");
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse();

    if !parse_result.errors.is_empty() {
        return false;
    }

    #[derive(Default)]
    struct TopLevelAwaitVisitor {
        depth: usize,
        found: bool,
    }

    impl<'a> Visit<'a> for TopLevelAwaitVisitor {
        fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
            self.depth += 1;
            walk_function(self, it, flags);
            self.depth = self.depth.saturating_sub(1);
        }

        fn visit_arrow_function_expression(
            &mut self,
            it: &oxc_ast::ast::ArrowFunctionExpression<'a>,
        ) {
            self.depth += 1;
            walk_arrow_function_expression(self, it);
            self.depth = self.depth.saturating_sub(1);
        }

        fn visit_await_expression(&mut self, _it: &oxc_ast::ast::AwaitExpression<'a>) {
            if self.depth == 1 {
                self.found = true;
            }
        }

        fn visit_for_of_statement(&mut self, it: &oxc_ast::ast::ForOfStatement<'a>) {
            if self.depth == 1 && it.r#await {
                self.found = true;
            }
            walk_for_of_statement(self, it);
        }
    }

    let mut visitor = TopLevelAwaitVisitor::default();
    visitor.visit_program(&parse_result.program);
    visitor.found
}
