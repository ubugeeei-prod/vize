//! Inline event handler processing.
//!
//! Transforms inline event handler expressions (e.g., `@click="count++"`)
//! by prefixing identifiers and wrapping in arrow functions when needed.

use vize_l0::{Box, String};

use crate::{ConstantType, ExpressionNode, SimpleExpressionNode, lane::TransformContext};

use super::{
    clone_expression, is_function_expression, normalize_expression,
    prefix::{get_identifier_prefix, is_simple_identifier},
    rewrite::{report_invalid_expression, rewrite_expression, rewrite_props_aliases},
    shape_checks::{is_event_handler_reference_node, is_function_expression_node},
    typescript::erase_typescript,
};

use vize_relief::JsExpression;

/// Process inline handler expression
pub fn process_inline_handler<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
) -> ExpressionNode<'a> {
    process_inline_handler_role(ctx, exp, false)
}

/// Process a compiler-generated `v-model` callback. Vue guards maybe-ref
/// writes only in this role; authored event handlers use normal assignment
/// expression lowering and write through `.value` directly.
pub(crate) fn process_model_handler<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
) -> ExpressionNode<'a> {
    process_inline_handler_role(ctx, exp, true)
}

fn process_inline_handler_role<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
    guard_maybe_ref_write: bool,
) -> ExpressionNode<'a> {
    let allocator = ctx.allocator;

    let normalized = normalize_expression(exp, allocator, ctx.source);

    if normalized.is_static || normalized.content.trim().is_empty() {
        return ExpressionNode::Simple(normalized);
    }

    // Skip if already processed for ref transformation
    if normalized.is_ref_transformed {
        return ExpressionNode::Simple(normalized);
    }

    let content = &normalized.content;
    // The retained AST applies wherever the checked text is still the node's
    // own bytes (P1-7); TS-erased text that changed falls back.
    let retained: Option<JsExpression<'_>> =
        crate::retained::retained_whole_expression(&normalized);
    let ts_stripped_content = ctx.options.is_ts.then(|| {
        erase_typescript(
            content,
            retained
                .as_ref()
                .is_some_and(crate::retained::js_module_compatible),
        )
    });
    let shape = ts_stripped_content
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(content);
    // The retained AST describes the authored bytes. Once erasure changed
    // them (`onPick!` → `onPick`, `onPick as any` → `onPick`), both the
    // function shape and the reference shape are read off that text.
    let shape_is_authored = shape == *content;
    let is_function = if shape_is_authored {
        is_function_expression_node(&normalized)
    } else {
        is_function_expression(shape)
    };
    let is_reference = if shape_is_authored {
        is_simple_identifier(content) || is_event_handler_reference_node(&normalized)
    } else {
        is_simple_identifier(shape) || super::is_event_handler_reference_expression(shape)
    };

    // Check if it's an inline function expression
    if is_function {
        // Process identifiers in the handler
        if ctx.options.prefix_identifiers {
            let result = rewrite_expression(
                content,
                ctx,
                false,
                retained.as_ref(),
                guard_maybe_ref_write,
            );
            if result.used_unref {
                ctx.helper(crate::RuntimeHelper::Unref);
            }
            if result.used_is_ref {
                ctx.helper(crate::RuntimeHelper::IsRef);
            }
            if let Some(detail) = &result.parse_error {
                report_invalid_expression(ctx, detail, &normalized.loc);
            }
            return ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode {
                    content: allocator.alloc_str(&result.code),
                    is_static: false,
                    const_type: ConstantType::NotConstant,
                    loc: normalized.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: true,
                    is_ref_transformed: true,
                },
                &allocator,
            ));
        } else if let Some(ts_stripped_content) = &ts_stripped_content {
            // Strip TypeScript type annotations even without prefix_identifiers
            return ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode {
                    content: allocator.alloc_str(ts_stripped_content),
                    is_static: false,
                    const_type: ConstantType::NotConstant,
                    loc: normalized.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: true,
                    is_ref_transformed: true,
                },
                &allocator,
            ));
        }
        return clone_expression(exp, allocator, ctx.source);
    }

    // Check if it's an identifier/member-expression handler reference.
    // Vue passes these directly without wrapping them in `$event => (...)`.
    // Codegen relies on this: a handler leaving this pass is either a function
    // expression or a reference, never a bare statement. The shape above
    // already unwraps TypeScript, so `onPick!` is the same reference as
    // `onPick` and the runtime calls it.
    if is_reference {
        let new_content: String = if ctx.options.prefix_identifiers {
            // Inline render functions read setup bindings directly, so an
            // identifier still needs its `_unref` / `.value` access.
            if is_simple_identifier(shape) && !ctx.options.inline {
                if let Some(prefix) = get_identifier_prefix(shape, ctx) {
                    let mut s = String::with_capacity(prefix.len() + shape.len());
                    s.push_str(prefix);
                    s.push_str(shape);
                    s
                } else {
                    shape.into()
                }
            } else {
                let result = rewrite_expression(
                    content,
                    ctx,
                    false,
                    retained.as_ref(),
                    guard_maybe_ref_write,
                );
                if result.used_unref {
                    ctx.helper(crate::RuntimeHelper::Unref);
                }
                if result.used_is_ref {
                    ctx.helper(crate::RuntimeHelper::IsRef);
                }
                if let Some(detail) = &result.parse_error {
                    report_invalid_expression(ctx, detail, &normalized.loc);
                }
                result.code
            }
        } else if let Some(erased) = &ts_stripped_content {
            erased.clone()
        } else {
            (*content).into()
        };

        let new_content = rewrite_props_aliases(new_content, ctx);

        return ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode {
                content: allocator.alloc_str(&new_content),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: normalized.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: true,
                is_ref_transformed: true,
            },
            &allocator,
        ));
    }

    // Compound expression - rewrite and wrap in arrow function
    let rewritten: String = if ctx.options.prefix_identifiers {
        // Only this branch introduces the implicit callback parameter. Method
        // references and authored functions retain their ordinary lexical scope.
        ctx.enter_scope(vize_croquis::ScopeKind::EventHandler);
        ctx.add_identifier("$event");
        let result = rewrite_expression(
            content,
            ctx,
            false,
            retained.as_ref(),
            guard_maybe_ref_write,
        );
        ctx.exit_scope();
        if result.used_unref {
            ctx.helper(crate::RuntimeHelper::Unref);
        }
        if result.used_is_ref {
            ctx.helper(crate::RuntimeHelper::IsRef);
        }
        if let Some(detail) = &result.parse_error {
            report_invalid_expression(ctx, detail, &normalized.loc);
        }
        result.code
    } else if let Some(erased) = ts_stripped_content {
        // Erase TypeScript even without prefix_identifiers
        erased
    } else {
        (*content).into()
    };
    // Use block body {...} for multi-statement handlers (semicolons),
    // concise body ( ... ) for single expressions. Vue emits the block body
    // with no surrounding spaces (`$event => {...}`).
    let new_content = if rewritten.contains(';') {
        let mut s = String::with_capacity(12 + rewritten.len() + 1);
        s.push_str("$event => {");
        s.push_str(&rewritten);
        s.push('}');
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
            content: allocator.alloc_str(&new_content),
            is_static: false,
            const_type: ConstantType::NotConstant,
            loc: normalized.loc.clone(),
            js_ast: None,
            hoisted: None,
            identifiers: None,
            is_handler_key: true,
            is_ref_transformed: true,
        },
        &allocator,
    ))
}

#[cfg(test)]
mod tests {
    use super::process_inline_handler;
    use crate::{
        CompoundExpressionNode, ExpressionNode, SourceLocation,
        lane::TransformContext,
        options::{BindingMetadata, BindingType, TransformOptions},
    };
    use vize_l0::{Allocator, Box, FxHashMap};

    fn test_context<'a>(allocator: &'a Allocator, source: &'a str) -> TransformContext<'a> {
        let mut bindings = FxHashMap::default();
        bindings.insert("selectedFolders".into(), BindingType::SetupRef);
        bindings.insert("folder".into(), BindingType::SetupRef);
        bindings.insert("compute".into(), BindingType::SetupConst);
        bindings.insert("emit".into(), BindingType::SetupConst);

        TransformContext::new(
            allocator,
            source,
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

    fn compound_expression<'a>(allocator: &'a Allocator, source: &str) -> ExpressionNode<'a> {
        let loc = SourceLocation::new(0, source.len() as u32);

        ExpressionNode::Compound(Box::new_in(
            CompoundExpressionNode::new(allocator, loc),
            &allocator,
        ))
    }

    #[test]
    fn test_process_inline_handler_rewrites_compound_ts_assignment() {
        let allocator = Allocator::new();
        let source = "selectedFolders = selectedFolders.filter(f => f.id !== folder!.id)";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_inline_handler(&mut ctx, &expr);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert!(result.content.starts_with("$event => ("));
        assert!(
            result
                .content
                .contains("selectedFolders.value = selectedFolders.value.filter(")
        );
        assert!(result.content.contains("folder.value.id"));
    }

    #[test]
    fn test_process_inline_handler_preserves_local_block_binding() {
        let allocator = Allocator::new();
        let source = "() => { const value = compute(); emit('change', value) }";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_inline_handler(&mut ctx, &expr);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert!(result.content.contains("const value = compute()"));
        assert!(result.content.contains("emit('change', value)"));
        assert!(!result.content.contains("_ctx.value"));
    }

    /// `onPick!` and `onPick as any` erase to the same identifier reference
    /// as `onPick`. The runtime calls that reference; a parenthesized
    /// `$event => (_ctx.onPick)` would only evaluate it.
    #[test]
    fn erased_handler_references_are_called_like_plain_identifiers() {
        for source in ["onPick!", "onPick as any", "onPick"] {
            let allocator = Allocator::new();
            let mut ctx = TransformContext::new(
                &allocator,
                source,
                TransformOptions {
                    prefix_identifiers: true,
                    is_ts: true,
                    ..Default::default()
                },
            );
            let expr = compound_expression(&allocator, source);
            let result = process_inline_handler(&mut ctx, &expr);
            let ExpressionNode::Simple(result) = result else {
                panic!("expected simple expression");
            };
            assert_eq!(result.content, "_ctx.onPick", "{source}");
        }
    }
}
