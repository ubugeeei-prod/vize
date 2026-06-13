//! Expression rewriting with identifier prefixing.
//!
//! Parses expressions with OXC, walks the AST to collect identifiers,
//! and applies prefix/suffix rewrites for proper context binding.

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use crate::SourceLocation;
use crate::errors::ErrorCode;
use crate::lane::TransformContext;

use super::{
    collector::IdentifierCollector,
    prefix::{get_identifier_prefix, is_ref_binding_simple, is_simple_identifier},
    typescript::strip_typescript_from_expression,
};

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn prop_access_expression(object: &str, key: &str) -> String {
    if is_simple_identifier(key) {
        let mut out = String::with_capacity(object.len() + key.len() + 1);
        out.push_str(object);
        out.push('.');
        out.push_str(key);
        return out;
    }

    let mut out = String::with_capacity(object.len() + key.len() + 4);
    out.push_str(object);
    out.push('[');
    use std::fmt::Write as _;
    let _ = write!(&mut out, "{:?}", key);
    out.push(']');
    out
}

fn replace_prefixed_alias_access(code: String, object: &str, local: &str, key: &str) -> String {
    let needle = {
        let mut needle = String::with_capacity(object.len() + local.len() + 1);
        needle.push_str(object);
        needle.push('.');
        needle.push_str(local);
        needle
    };
    let replacement = prop_access_expression(object, key);

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0;
    while let Some(rel_pos) = code[cursor..].find(needle.as_str()) {
        let start = cursor + rel_pos;
        let end = start + needle.len();
        let after_ok = code[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_identifier_continue(c));

        result.push_str(&code[cursor..start]);
        if after_ok {
            result.push_str(&replacement);
        } else {
            result.push_str(&code[start..end]);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

pub(super) fn rewrite_props_aliases(code: String, ctx: &TransformContext<'_>) -> String {
    let Some(bindings) = &ctx.options.binding_metadata else {
        return code;
    };
    if bindings.props_aliases.is_empty() {
        return code;
    }

    let mut rewritten = code;
    for (local, key) in &bindings.props_aliases {
        rewritten = replace_prefixed_alias_access(rewritten, "__props", local, key);
        rewritten = replace_prefixed_alias_access(rewritten, "$props", local, key);
    }
    rewritten
}

/// Result of expression rewriting
pub(crate) struct RewriteResult {
    pub(crate) code: String,
    pub(crate) used_unref: bool,
    /// Set when the expression could not be parsed at all and the raw
    /// content was passed through. Holds the parser's error detail so the
    /// caller can emit a compile diagnostic (mirroring `@vue/compiler-core`'s
    /// `X_INVALID_EXPRESSION`). `None` on every successful rewrite path.
    pub(crate) parse_error: Option<String>,
}

/// Emit an `InvalidExpression` compile diagnostic for an expression that
/// failed to parse, matching `@vue/compiler-core`'s
/// `Error parsing JavaScript expression: <detail>` message format.
pub(super) fn report_invalid_expression(
    ctx: &mut TransformContext<'_>,
    detail: &str,
    loc: &SourceLocation,
) {
    const PREFIX: &str = "Error parsing JavaScript expression: ";
    let mut message = String::with_capacity(PREFIX.len() + detail.len());
    message.push_str(PREFIX);
    message.push_str(detail);
    ctx.on_error_with_message(ErrorCode::InvalidExpression, message, Some(loc.clone()));
}

/// Returns true when `content` parses as a TypeScript expression or program.
///
/// Only consulted on the parse-failure path for `is_ts` templates: when the
/// TypeScript-stripping step falls back to the original source, the plain-JS
/// parse below can fail even though the expression is valid TypeScript that
/// the official compiler (babel with the `typescript` plugin) accepts. The
/// parity rule is that vize must not reject what the official compiler
/// accepts, so such expressions keep the silent passthrough behavior.
fn parses_as_typescript(content: &str) -> bool {
    let source_type = SourceType::ts().with_module(true);

    let expr_allocator = OxcAllocator::default();
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    if Parser::new(&expr_allocator, &wrapped, source_type)
        .parse_expression()
        .is_ok()
    {
        return true;
    }

    let program_allocator = OxcAllocator::default();
    Parser::new(&program_allocator, content, source_type)
        .parse()
        .errors
        .is_empty()
}

/// Rewrite an expression string, prefixing identifiers with `_ctx.` where needed
pub(crate) fn rewrite_expression(
    content: &str,
    ctx: &TransformContext<'_>,
    _as_params: bool,
) -> RewriteResult {
    // Skip parsing for inputs that would overflow the parser stack — return
    // the original content unchanged so the compile lane emits a normal
    // diagnostic for the surrounding directive instead of aborting. (#956)
    if super::expression_exceeds_max_depth(content) {
        return RewriteResult {
            code: String::new(content),
            used_unref: false,
            parse_error: None,
        };
    }
    // First, if this is TypeScript, strip type annotations
    let js_content = if ctx.options.is_ts {
        strip_typescript_from_expression(content)
    } else {
        String::new(content)
    };

    // Try to parse as a JavaScript expression
    let oxc_allocator = OxcAllocator::default();
    let source_type = SourceType::default().with_module(true);

    // Wrap in parentheses to make it a valid expression statement
    let mut wrapped = String::with_capacity(js_content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(&js_content);
    wrapped.push(')');
    let parser = Parser::new(&oxc_allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => {
            // Successfully parsed - walk the AST and collect identifiers to rewrite
            let mut collector = IdentifierCollector::new(ctx, &wrapped);
            collector.visit_expression(&expr);

            let used_unref = collector.used_unref;

            // Combine prefix rewrites (from HashSet) with suffix rewrites
            // Each rewrite is (position, prefix, suffix)
            let mut all_rewrites: Vec<(usize, String, String)> = collector
                .rewrites
                .into_iter()
                .map(|(pos, prefix)| (pos, prefix, String::default()))
                .collect();

            // Add suffix rewrites (suffixes come after the identifier)
            for (pos, suffix) in collector.suffix_rewrites {
                all_rewrites.push((pos, String::default(), suffix));
            }

            // Sort by position descending so we can replace from end to start
            all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

            // Apply rewrites
            let mut result = js_content.clone();
            for (pos, prefix, suffix) in all_rewrites {
                // Adjust position for the wrapping parenthesis we added
                let adjusted_pos = pos.saturating_sub(1);
                if adjusted_pos <= result.len() {
                    if !suffix.is_empty() {
                        // Insert suffix at the end of identifier
                        result.insert_str(adjusted_pos, &suffix);
                    }
                    if !prefix.is_empty() {
                        // Insert prefix at the start of identifier
                        result.insert_str(adjusted_pos, &prefix);
                    }
                }
            }

            RewriteResult {
                code: rewrite_props_aliases(result, ctx),
                used_unref,
                parse_error: None,
            }
        }
        Err(expression_errors) => {
            // Expression parsing failed - try parsing as a program (multi-statement handlers)
            let oxc_allocator2 = OxcAllocator::default();
            let parser2 = Parser::new(&oxc_allocator2, &js_content, source_type);
            let parse_result2 = parser2.parse();

            if parse_result2.errors.is_empty() {
                // Successfully parsed as program - walk the AST and collect identifiers
                let mut collector = IdentifierCollector::new(ctx, &js_content);
                collector.visit_program(&parse_result2.program);

                let used_unref = collector.used_unref;

                let mut all_rewrites: Vec<(usize, String, String)> = collector
                    .rewrites
                    .into_iter()
                    .map(|(pos, prefix)| (pos, prefix, String::default()))
                    .collect();

                for (pos, suffix) in collector.suffix_rewrites {
                    all_rewrites.push((pos, String::default(), suffix));
                }

                all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

                let mut result = js_content.clone();
                for (pos, prefix, suffix) in all_rewrites {
                    // No offset adjustment needed - program parsing has no wrapping parens
                    if pos <= result.len() {
                        if !suffix.is_empty() {
                            result.insert_str(pos, &suffix);
                        }
                        if !prefix.is_empty() {
                            result.insert_str(pos, &prefix);
                        }
                    }
                }

                return RewriteResult {
                    code: rewrite_props_aliases(result, ctx),
                    used_unref,
                    parse_error: None,
                };
            }

            // Program parsing also failed - fallback to simple identifier check
            let mut parse_error = None;
            let code: String = if is_simple_identifier(&js_content) {
                // Reserved words (`class`, `default`, …) fail to parse as an
                // expression but are still rewritable identifiers. Vue treats
                // them through its simple-identifier fast path without ever
                // parsing, so no diagnostic is emitted here either.
                if let Some(prefix) = get_identifier_prefix(&js_content, ctx) {
                    let mut s = String::with_capacity(prefix.len() + js_content.len());
                    s.push_str(prefix);
                    s.push_str(&js_content);
                    s
                } else if is_ref_binding_simple(&js_content, ctx) {
                    // Add .value for refs in inline mode
                    let mut s = String::with_capacity(js_content.len() + 6);
                    s.push_str(&js_content);
                    s.push_str(".value");
                    s
                } else {
                    js_content
                }
            } else {
                // The raw content is passed through unprefixed. The official
                // compiler reports `X_INVALID_EXPRESSION` here, so surface the
                // parser detail for the caller to emit a diagnostic — unless
                // the original source is valid TypeScript that only vize's
                // TS-stripping fallback failed to lower (the official compiler
                // accepts it, so vize must not reject it).
                if !ctx.options.is_ts || !parses_as_typescript(content) {
                    parse_error = Some(
                        expression_errors
                            .first()
                            .map(|error| String::new(error.message.as_ref()))
                            .unwrap_or_else(|| String::new("invalid expression")),
                    );
                }
                js_content
            };
            RewriteResult {
                code: rewrite_props_aliases(code, ctx),
                used_unref: false,
                parse_error,
            }
        }
    }
}
