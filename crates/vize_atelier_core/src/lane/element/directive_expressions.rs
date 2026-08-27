//! `_ctx`-prefixing of the expressions carried by an element's directives.
//!
//! Split out of [`super::element`] so that module stays inside the per-file
//! source-length budget.

use vize_s0::Box;

use crate::steps::expression::{process_expression, process_inline_handler};
use crate::{ElementNode, ExpressionNode, PropNode};

use super::TransformContext;

/// Process directive expressions with _ctx prefix
pub(super) fn process_directive_expressions<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) {
    for prop in el.props.iter_mut() {
        if let PropNode::Directive(dir) = prop {
            match dir.name {
                "bind" | "show" | "if" | "else-if" | "for" | "memo" => {
                    // Process value expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                }
                "on" => {
                    if let Some(exp) = &dir.exp {
                        if dir.arg.is_none() {
                            // v-on="obj" - process as regular expression (object of handlers),
                            // NOT as an inline handler. toHandlers() expects an object, not a function.
                            let processed = process_expression(ctx, exp, false);
                            dir.exp = Some(processed);
                        } else {
                            // v-on:event="handler" - process as inline handler
                            let processed = process_inline_handler(ctx, exp);
                            dir.exp = Some(processed);
                        }
                    }
                }
                "model" => {
                    // Process v-model expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                    // A dynamic argument (`v-model:[prop]`, or Babel JSX's
                    // `v-model={[value, prop]}`) is an expression too, and
                    // codegen emits it verbatim, so it has to be prefixed here
                    // exactly like the value is.
                    if let Some(arg) = &dir.arg
                        && let ExpressionNode::Simple(simple_arg) = arg
                        && !simple_arg.is_static
                    {
                        let processed = process_expression(ctx, arg, false);
                        dir.arg = Some(processed);
                    }
                }
                "slot" => {
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, true);
                        dir.exp = Some(processed);
                    }
                    if let Some(arg) = &dir.arg
                        && let ExpressionNode::Simple(simple_arg) = arg
                        && !simple_arg.is_static
                    {
                        let processed = process_expression(ctx, arg, false);
                        dir.arg = Some(processed);
                    }
                }
                _ => {
                    // Custom directives - process value expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                    // Process dynamic argument
                    if let Some(arg) = &dir.arg
                        && let ExpressionNode::Simple(simple_arg) = arg
                        && !simple_arg.is_static
                    {
                        let processed = process_expression(ctx, arg, false);
                        dir.arg = Some(processed);
                    }
                }
            }
        }
    }
}
