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
    clone_expression, is_event_handler_reference_expression, is_function_expression,
    normalize_expression,
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

    let normalized = normalize_expression(exp, allocator);

    if normalized.is_static {
        return ExpressionNode::Simple(normalized);
    }

    // Skip if already processed for ref transformation
    if normalized.is_ref_transformed {
        return ExpressionNode::Simple(normalized);
    }

    let content = &normalized.content;
    let ts_stripped_content = if ctx.options.is_ts {
        Some(strip_typescript_from_expression(content))
    } else {
        None
    };
    let function_check_source = ts_stripped_content
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(content.as_str());

    // Check if it's an inline function expression
    if is_function_expression(function_check_source) {
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
                    loc: normalized.loc.clone(),
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
            return ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode {
                    content: String::new(ts_stripped_content.as_ref().unwrap()),
                    is_static: false,
                    const_type: ConstantType::NotConstant,
                    loc: normalized.loc.clone(),
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
                loc: normalized.loc.clone(),
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
            loc: normalized.loc.clone(),
            js_ast: None,
            hoisted: None,
            identifiers: None,
            is_handler_key: true,
            is_ref_transformed: true,
        },
        allocator,
    ))
}

#[cfg(test)]
mod tests {
    use super::process_inline_handler;
    use crate::{
        ast::{CompoundExpressionNode, ExpressionNode, Position, SourceLocation},
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
    fn test_process_inline_handler_rewrites_compound_ts_assignment() {
        let allocator = Bump::new();
        let mut ctx = test_context(&allocator);
        let expr = compound_expression(
            &allocator,
            "selectedFolders = selectedFolders.filter(f => f.id !== folder!.id)",
        );

        let result = process_inline_handler(&mut ctx, &expr);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert!(result.content.starts_with("$event => ("));
        assert!(result
            .content
            .contains("selectedFolders.value = selectedFolders.value.filter("));
        assert!(result.content.contains("folder.value.id"));
    }
}
