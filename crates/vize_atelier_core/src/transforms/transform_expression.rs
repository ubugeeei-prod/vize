//! Expression transform.
//!
//! Transforms expressions by prefixing identifiers with `_ctx.` for proper
//! context binding in the compiled render function (script setup mode).

mod collector;
mod inline_handler;
pub(crate) mod prefix;
mod rewrite;
mod typescript;

use oxc_ast::ast::{ChainElement, Expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{Box, Bump, String};

/// Maximum bracket nesting depth allowed in a template expression.
///
/// Mirrors the parser's `MAX_ELEMENT_NESTING_DEPTH = 256`, but for the JS/TS
/// expression text inside `{{ … }}` / directive values. The oxc parser
/// recurses for each `(` / `[` / `{` it sees, so a sufficiently nested
/// expression overflows the native stack and aborts the process — a stack
/// overflow cannot be caught with `catch_unwind`. The same expression
/// path is shared by `vize build`, `vize check`, the LSP, the linter,
/// the formatter, and the Vite dev-server middleware. (#956)
pub const MAX_EXPRESSION_NESTING_DEPTH: usize = 256;

/// Returns the maximum bracket nesting depth in `content`. Only counts
/// `(`, `[`, `{` and their closers — these are what drive recursion in the
/// JS parser and the recursive AST walkers in `prefix` / `rewrite`. Strings
/// and template literals are skipped so contents like `"((((((((..."` don't
/// trigger a false positive.
pub fn expression_nesting_depth(content: &str) -> usize {
    let bytes = content.as_bytes();
    let mut depth: usize = 0;
    let mut max_depth: usize = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'"' | b'\'' | b'`' => {
                let quote = b;
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i = i.saturating_add(2);
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = i.saturating_add(2);
                continue;
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    max_depth
}

/// Returns true if `content` exceeds [`MAX_EXPRESSION_NESTING_DEPTH`].
#[inline]
pub fn expression_exceeds_max_depth(content: &str) -> bool {
    expression_nesting_depth(content) > MAX_EXPRESSION_NESTING_DEPTH
}

use crate::{
    ast::{ConstantType, ExpressionNode, SimpleExpressionNode},
    transform::TransformContext,
};

pub use inline_handler::process_inline_handler;
pub use prefix::{is_simple_identifier, prefix_identifiers_in_expression};
pub use typescript::strip_typescript_from_expression;

use rewrite::rewrite_expression;

/// Returns true if an expression is a callable reference that should be passed
/// through directly as an event handler, not wrapped as `$event => (...)`.
pub fn is_event_handler_reference_expression(content: &str) -> bool {
    if expression_exceeds_max_depth(content) {
        return false;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parser = Parser::new(&allocator, content, SourceType::default().with_module(true));
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };

    match expr {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::ChainExpression(chain) => matches!(
            chain.expression,
            ChainElement::StaticMemberExpression(_) | ChainElement::ComputedMemberExpression(_)
        ),
        _ => false,
    }
}

/// Returns true if the whole expression is a function / arrow function expression.
pub fn is_function_expression(content: &str) -> bool {
    if expression_exceeds_max_depth(content) {
        return false;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parser = Parser::new(&allocator, content, SourceType::default().with_module(true));
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };

    matches!(
        expr,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

/// Process expression with identifier prefixing and TypeScript stripping
pub fn process_expression<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
    as_params: bool,
) -> ExpressionNode<'a> {
    let allocator = ctx.allocator;

    let normalized = normalize_expression(exp, allocator);

    // If not prefixing identifiers and not TypeScript, just clone
    if !ctx.options.prefix_identifiers && !ctx.options.is_ts {
        return ExpressionNode::Simple(normalized);
    }

    if normalized.is_static {
        return ExpressionNode::Simple(normalized);
    }

    // Skip if already processed for ref transformation
    if normalized.is_ref_transformed {
        return ExpressionNode::Simple(normalized);
    }

    let content = &normalized.content;

    // Empty content
    if content.is_empty() {
        return ExpressionNode::Simple(normalized);
    }

    // Strip TypeScript if needed, then optionally prefix identifiers
    let processed = if ctx.options.prefix_identifiers {
        // rewrite_expression handles both TS stripping and prefixing
        let result = rewrite_expression(content, ctx, as_params);
        if result.used_unref {
            ctx.helper(crate::ast::RuntimeHelper::Unref);
        }
        result.code
    } else if ctx.options.is_ts {
        // Only strip TypeScript, no prefixing
        strip_typescript_from_expression(content)
    } else {
        String::new(content)
    };

    ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode {
            content: processed,
            is_static: false,
            const_type: normalized.const_type,
            loc: normalized.loc.clone(),
            js_ast: None,
            hoisted: None,
            identifiers: None,
            is_handler_key: normalized.is_handler_key,
            is_ref_transformed: true,
        },
        allocator,
    ))
}

/// Clone an expression node.
///
/// Compound expressions are flattened to a [`SimpleExpressionNode`] whose
/// content is the original source text. This mirrors the strategy used by
/// [`normalize_expression`] and `transform::context::clone_expression`,
/// and avoids the previous behavior of producing a `Compound` node with
/// an empty `children` list (which silently dropped the expression).
pub(crate) fn clone_expression<'a>(
    exp: &ExpressionNode<'a>,
    allocator: &'a Bump,
) -> ExpressionNode<'a> {
    match exp {
        ExpressionNode::Simple(simple) => ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode {
                content: simple.content.clone(),
                is_static: simple.is_static,
                const_type: simple.const_type,
                loc: simple.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: simple.is_handler_key,
                is_ref_transformed: simple.is_ref_transformed,
            },
            allocator,
        )),
        ExpressionNode::Compound(compound) => ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode {
                content: compound.loc.source.clone(),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: compound.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: compound.is_handler_key,
                is_ref_transformed: false,
            },
            allocator,
        )),
    }
}

pub(crate) fn normalize_expression<'a>(
    exp: &ExpressionNode<'a>,
    allocator: &'a Bump,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => Box::new_in(
            SimpleExpressionNode {
                content: simple.content.clone(),
                is_static: simple.is_static,
                const_type: simple.const_type,
                loc: simple.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: simple.is_handler_key,
                is_ref_transformed: simple.is_ref_transformed,
            },
            allocator,
        ),
        ExpressionNode::Compound(compound) => Box::new_in(
            SimpleExpressionNode {
                content: compound.loc.source.clone(),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: compound.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: compound.is_handler_key,
                is_ref_transformed: false,
            },
            allocator,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_EXPRESSION_NESTING_DEPTH, clone_expression, expression_exceeds_max_depth,
        expression_nesting_depth, is_event_handler_reference_expression, is_function_expression,
        prefix::prefix_identifiers_in_expression, process_expression,
        typescript::strip_typescript_from_expression,
    };
    use crate::{
        ast::{CompoundExpressionNode, ExpressionNode, Position, RuntimeHelper, SourceLocation},
        options::{BindingMetadata, BindingType, TransformOptions},
        transform::TransformContext,
    };
    use vize_carton::{Box, Bump, FxHashMap};

    fn test_context<'a>(allocator: &'a Bump) -> TransformContext<'a> {
        let mut bindings = FxHashMap::default();
        bindings.insert("selectedFolders".into(), BindingType::SetupRef);
        bindings.insert("folder".into(), BindingType::SetupRef);

        TransformContext::new(
            allocator,
            "".into(),
            TransformOptions {
                prefix_identifiers: true,
                inline: true,
                is_ts: true,
                binding_metadata: Some(BindingMetadata {
                    bindings,
                    props_aliases: FxHashMap::default(),
                    is_script_setup: true,
                }),
                ..Default::default()
            },
        )
    }

    fn compound_expression<'a>(allocator: &'a Bump, source: &str) -> ExpressionNode<'a> {
        let loc = SourceLocation::new(
            Position::new(0, 1, 1),
            Position::new(source.len() as u32, 1, source.len() as u32 + 1),
            source,
        );

        ExpressionNode::Compound(Box::new_in(
            CompoundExpressionNode::new(allocator, loc),
            allocator,
        ))
    }

    #[test]
    fn test_process_expression_rewrites_compound_ts_ref_reads() {
        let allocator = Bump::new();
        let mut ctx = test_context(&allocator);
        let expr = compound_expression(
            &allocator,
            "!selectedFolders.some(f => f.id === folder!.id)",
        );

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert!(result.content.starts_with("!selectedFolders.value.some("));
        assert!(result.content.contains("folder.value.id"));
    }

    #[test]
    fn test_process_expression_uses_setup_proxy_in_function_mode() {
        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("isExternal".into(), BindingType::SetupRef);

        let mut ctx = TransformContext::new(
            &allocator,
            "".into(),
            TransformOptions {
                prefix_identifiers: true,
                inline: false,
                is_ts: true,
                binding_metadata: Some(BindingMetadata {
                    bindings,
                    props_aliases: FxHashMap::default(),
                    is_script_setup: true,
                }),
                ..Default::default()
            },
        );
        let expr = compound_expression(&allocator, "isExternal && isExternal.value");

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert_eq!(
            result.content.as_str(),
            "$setup.isExternal && $setup.isExternal.value"
        );
        assert!(!ctx.has_helper(RuntimeHelper::Unref));
    }

    #[test]
    fn test_expression_nesting_depth_counts_parens() {
        assert_eq!(expression_nesting_depth("a + b"), 0);
        assert_eq!(expression_nesting_depth("(a + b)"), 1);
        assert_eq!(expression_nesting_depth("((a + b))"), 2);
        assert_eq!(expression_nesting_depth("[[[1]]]"), 3);
        assert_eq!(expression_nesting_depth("{a: 1}"), 1);
    }

    #[test]
    fn test_expression_nesting_depth_ignores_brackets_in_strings_and_comments() {
        assert_eq!(expression_nesting_depth(r#""((((""#), 0);
        assert_eq!(expression_nesting_depth(r#"'((((((' + 1"#), 0);
        assert_eq!(expression_nesting_depth("`((((`"), 0);
        assert_eq!(expression_nesting_depth("a /* (((( */ b"), 0);
        assert_eq!(expression_nesting_depth("a // ((((\n + b"), 0);
    }

    #[test]
    fn test_expression_exceeds_max_depth_guards_deeply_nested() {
        let deep = "(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)
            + "1"
            + &")".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
        assert!(expression_exceeds_max_depth(&deep));
        let shallow = "(".repeat(MAX_EXPRESSION_NESTING_DEPTH)
            + "1"
            + &")".repeat(MAX_EXPRESSION_NESTING_DEPTH);
        assert!(!expression_exceeds_max_depth(&shallow));
    }

    #[test]
    fn test_expression_entry_points_do_not_overflow_on_deep_input() {
        // Regression for #956: every entry point that previously fed the
        // recursive oxc parser must return a benign value for an input
        // beyond MAX_EXPRESSION_NESTING_DEPTH rather than abort the
        // process via stack overflow.
        let deep = "(".repeat(100_000) + "1" + &")".repeat(100_000);
        assert!(!is_event_handler_reference_expression(&deep));
        assert!(!is_function_expression(&deep));
        let prefixed = prefix_identifiers_in_expression(&deep);
        assert_eq!(prefixed.as_str(), deep.as_str());
        let stripped = strip_typescript_from_expression(&deep);
        assert_eq!(stripped.as_str(), deep.as_str());
    }

    #[test]
    fn test_clone_expression_preserves_compound_source() {
        let allocator = Bump::new();
        let source = "foo + bar";
        let expr = compound_expression(&allocator, source);

        let cloned = clone_expression(&expr, &allocator);
        let ExpressionNode::Simple(simple) = cloned else {
            panic!("expected clone_expression to flatten Compound to Simple");
        };

        assert_eq!(simple.content.as_str(), source);
    }
}

// Note: Multiline arrow function handling and ES6 shorthand expansion
// are tested via SFC snapshot tests in tests/fixtures/sfc/patches.toml.
