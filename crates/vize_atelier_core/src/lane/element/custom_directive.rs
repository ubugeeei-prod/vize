//! Custom directive registration and declared contracts (#13074).
//!
//! A directive declared by a `vue.plugins` entry has a modifier contract.
//! The transform checks each binding against it once and appends the
//! absent modifiers that default to `true`, so every legacy consumer (DOM
//! and SSR fallbacks, Vapor) emits the same normalized modifiers the L2 lane
//! does. A binding that breaks the contract is a compile error on its span.

use vize_l0::{Box, cstr};

use crate::errors::ErrorCode;
use crate::{DirectiveNode, RuntimeHelper, SimpleExpressionNode, SourceLocation};

use super::TransformContext;

/// Register `dir` for `resolveDirective` and apply its declared contract.
pub(super) fn register<'a>(ctx: &mut TransformContext<'a>, dir: &mut Box<'a, DirectiveNode<'a>>) {
    ctx.helper(RuntimeHelper::WithDirectives);
    ctx.helper(RuntimeHelper::ResolveDirective);
    ctx.add_directive(dir.name);
    if ctx.options.directive_contracts.is_empty() {
        return;
    }
    let contracts = ctx.options.directive_contracts.clone();
    let Some(contract) = contracts.get(dir.name) else {
        return;
    };
    let authored: std::vec::Vec<&str> = dir
        .modifiers
        .iter()
        .map(|modifier| modifier.content)
        .collect();
    match contract.default_modifiers(&authored) {
        Ok(defaults) => {
            let allocator = ctx.allocator;
            for name in defaults {
                let content = allocator.alloc_str(name);
                dir.modifiers.push(SimpleExpressionNode::new(
                    content,
                    true,
                    SourceLocation::STUB,
                ));
            }
        }
        Err(error) => {
            let message = cstr!("{error} (declared by {})", contract.plugin);
            let loc = dir.loc.clone();
            ctx.on_error_with_message(ErrorCode::DirectiveContract, message, Some(loc));
        }
    }
}
