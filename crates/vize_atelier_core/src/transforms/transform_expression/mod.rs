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

use crate::{
    ast::{CompoundExpressionNode, ExpressionNode, SimpleExpressionNode},
    transform::TransformContext,
};

pub use inline_handler::process_inline_handler;
pub use prefix::{is_simple_identifier, prefix_identifiers_in_expression};
pub use typescript::strip_typescript_from_expression;

use rewrite::rewrite_expression;

/// Returns true if an expression is a callable reference that should be passed
/// through directly as an event handler, not wrapped as `$event => (...)`.
pub fn is_event_handler_reference_expression(content: &str) -> bool {
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

/// Process expression with identifier prefixing and TypeScript stripping
pub fn process_expression<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
    as_params: bool,
) -> ExpressionNode<'a> {
    let allocator = ctx.allocator;

    // If not prefixing identifiers and not TypeScript, just clone
    if !ctx.options.prefix_identifiers && !ctx.options.is_ts {
        return clone_expression(exp, allocator);
    }

    match exp {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                return clone_expression(exp, allocator);
            }

            // Skip if already processed for ref transformation
            if simple.is_ref_transformed {
                return clone_expression(exp, allocator);
            }

            let content = &simple.content;

            // Empty content
            if content.is_empty() {
                return clone_expression(exp, allocator);
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
                    const_type: simple.const_type,
                    loc: simple.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: simple.is_handler_key,
                    is_ref_transformed: true,
                },
                allocator,
            ))
        }
        ExpressionNode::Compound(_compound) => {
            // For compound expressions, process each child
            clone_expression(exp, allocator)
        }
    }
}

/// Clone an expression node
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
        ExpressionNode::Compound(compound) => {
            // TODO: proper compound expression cloning
            ExpressionNode::Compound(Box::new_in(
                CompoundExpressionNode {
                    children: bumpalo::collections::Vec::new_in(allocator),
                    loc: compound.loc.clone(),
                    identifiers: None,
                    is_handler_key: compound.is_handler_key,
                },
                allocator,
            ))
        }
    }
}

// Note: Multiline arrow function handling and ES6 shorthand expansion
// are tested via SFC snapshot tests in tests/fixtures/sfc/patches.toml.
