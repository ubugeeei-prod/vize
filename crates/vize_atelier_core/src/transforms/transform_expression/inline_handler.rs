//! Inline event handler processing.
//!
//! Transforms inline event handler expressions (e.g., `@click="count++"`)
//! by prefixing identifiers and wrapping in arrow functions when needed.

use vize_carton::{Box, String};

use crate::{
    ast::{ConstantType, ExpressionNode, SimpleExpressionNode},
    transform::TransformContext,
};

use super::{
    clone_expression, is_event_handler_reference_expression,
    prefix::{get_identifier_prefix, is_simple_identifier},
    rewrite::rewrite_expression,
    typescript::strip_typescript_from_expression,
};

/// Process inline handler expression
pub fn process_inline_handler<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
) -> ExpressionNode<'a> {
    let allocator = ctx.allocator;

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

            // Check if it's an inline function expression
            if content.contains("=>") || content.starts_with("function") {
                // Process identifiers in the handler
                if ctx.options.prefix_identifiers {
                    let result = rewrite_expression(content, ctx, false);
                    if result.used_unref {
                        ctx.helper(crate::ast::RuntimeHelper::Unref);
                    }
                    return ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode {
                            content: String::new(&result.code),
                            is_static: false,
                            const_type: ConstantType::NotConstant,
                            loc: simple.loc.clone(),
                            js_ast: None,
                            hoisted: None,
                            identifiers: None,
                            is_handler_key: true,
                            is_ref_transformed: true,
                        },
                        allocator,
                    ));
                } else if ctx.options.is_ts {
                    // Strip TypeScript type annotations even without prefix_identifiers
                    let stripped = strip_typescript_from_expression(content);
                    return ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode {
                            content: String::new(&stripped),
                            is_static: false,
                            const_type: ConstantType::NotConstant,
                            loc: simple.loc.clone(),
                            js_ast: None,
                            hoisted: None,
                            identifiers: None,
                            is_handler_key: true,
                            is_ref_transformed: true,
                        },
                        allocator,
                    ));
                }
                return clone_expression(exp, allocator);
            }

            // Check if it's an identifier/member-expression handler reference.
            // Vue passes these directly without wrapping them in `$event => (...)`.
            if is_simple_identifier(content) || is_event_handler_reference_expression(content) {
                let new_content: String = if ctx.options.prefix_identifiers {
                    if is_simple_identifier(content) {
                        if let Some(prefix) = get_identifier_prefix(content, ctx) {
                            let mut s = String::with_capacity(prefix.len() + content.len());
                            s.push_str(prefix);
                            s.push_str(content);
                            s
                        } else {
                            content.clone()
                        }
                    } else {
                        let result = rewrite_expression(content, ctx, false);
                        if result.used_unref {
                            ctx.helper(crate::ast::RuntimeHelper::Unref);
                        }
                        result.code
                    }
                } else if ctx.options.is_ts {
                    strip_typescript_from_expression(content)
                } else {
                    content.clone()
                };

                return ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode {
                        content: new_content,
                        is_static: false,
                        const_type: ConstantType::NotConstant,
                        loc: simple.loc.clone(),
                        js_ast: None,
                        hoisted: None,
                        identifiers: None,
                        is_handler_key: true,
                        is_ref_transformed: true,
                    },
                    allocator,
                ));
            }

            // Compound expression - rewrite and wrap in arrow function
            let rewritten: String = if ctx.options.prefix_identifiers {
                let result = rewrite_expression(content, ctx, false);
                if result.used_unref {
                    ctx.helper(crate::ast::RuntimeHelper::Unref);
                }
                result.code
            } else if ctx.options.is_ts {
                // Strip TypeScript type annotations even without prefix_identifiers
                strip_typescript_from_expression(content)
            } else {
                content.clone()
            };
            // Use block body { ... } for multi-statement handlers (semicolons),
            // concise body ( ... ) for single expressions
            let new_content = if rewritten.contains(';') {
                let mut s = String::with_capacity(14 + rewritten.len() + 2);
                s.push_str("$event => { ");
                s.push_str(&rewritten);
                s.push_str(" }");
                s
            } else {
                let mut s = String::with_capacity(12 + rewritten.len() + 1);
                s.push_str("$event => (");
                s.push_str(&rewritten);
                s.push(')');
                s
            };

            ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode {
                    content: new_content,
                    is_static: false,
                    const_type: ConstantType::NotConstant,
                    loc: simple.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: true,
                    is_ref_transformed: true,
                },
                allocator,
            ))
        }
        _ => clone_expression(exp, allocator),
    }
}
