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
    ast::{CompoundExpressionNode, ConstantType, ExpressionNode, SimpleExpressionNode},
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

/// Returns true if the whole expression is a function / arrow function expression.
pub fn is_function_expression(content: &str) -> bool {
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
    use super::process_expression;
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
    fn test_process_expression_unrefs_function_mode_setup_refs() {
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
            "_unref($setup.isExternal) && $setup.isExternal.value"
        );
        assert!(ctx.has_helper(RuntimeHelper::Unref));
    }
}

// Note: Multiline arrow function handling and ES6 shorthand expansion
// are tested via SFC snapshot tests in tests/fixtures/sfc/patches.toml.
