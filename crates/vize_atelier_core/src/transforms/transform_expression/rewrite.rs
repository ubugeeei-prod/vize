//! Expression rewriting with identifier prefixing.
//!
//! Parses expressions with OXC, walks the AST to collect identifiers,
//! and applies prefix/suffix rewrites for proper context binding.

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use crate::transform::TransformContext;

use super::{
    collector::IdentifierCollector,
    prefix::{get_identifier_prefix, is_ref_binding_simple, is_simple_identifier},
    typescript::strip_typescript_from_expression,
};

/// Result of expression rewriting
pub(crate) struct RewriteResult {
    pub(crate) code: String,
    pub(crate) used_unref: bool,
}

/// Rewrite an expression string, prefixing identifiers with `_ctx.` where needed
pub(crate) fn rewrite_expression(
    content: &str,
    ctx: &TransformContext<'_>,
    _as_params: bool,
) -> RewriteResult {
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
                code: result,
                used_unref,
            }
        }
        Err(_) => {
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
                    code: result,
                    used_unref,
                };
            }

            // Program parsing also failed - fallback to simple identifier check
            let code: String = if is_simple_identifier(&js_content) {
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
                js_content
            };
            RewriteResult {
                code,
                used_unref: false,
            }
        }
    }
}
