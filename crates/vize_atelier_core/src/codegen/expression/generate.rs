//! Event handler and expression generation.
//!
//! Handles generating event handler expressions with proper arrow function
//! wrapping, TypeScript stripping, and identifier prefixing.

use crate::{CompoundExpressionChild, ExpressionNode};

use super::{
    super::context::CodegenContext,
    generate_simple_expression,
    prefix_context::{prefix_identifiers_with_context, prefix_identifiers_with_context_node},
    scope_prefix::{contains_slot_param_scope_prefix, strip_scope_prefixes_for_slot_params},
};
use vize_s0::String;

/// Generate a simple expression with appropriate prefix.
/// Used for ref attribute values that need `$setup.` prefix in function mode.
#[allow(dead_code)]
pub fn generate_simple_expression_with_prefix(ctx: &CodegenContext, content: &str) -> String {
    prefix_identifiers_with_context(content, ctx)
}

/// Check if a string is a member-expression style handler reference.
/// This includes forms like `_ctx.foo`, `$setup.bar`, and `_unref(store).save`.
pub fn is_simple_member_expression(s: &str) -> bool {
    crate::steps::is_event_handler_reference_expression(s)
}

/// Check if an event handler expression is an inline handler.
/// Inline handlers are expressions that are NOT simple identifiers or member expressions.
#[allow(dead_code)]
pub fn is_inline_handler(ctx: &CodegenContext, exp: &ExpressionNode<'_>) -> bool {
    match exp {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                return false;
            }

            let content = simple.loc.span.slice(&ctx.source);

            if crate::steps::expression::is_function_expression(content) {
                return false;
            }

            if crate::steps::is_simple_identifier(content) || is_simple_member_expression(content) {
                return false;
            }

            true
        }
        ExpressionNode::Compound(_) => true,
    }
}

/// Generate event handler expression.
///
/// Wraps inline expressions in arrow functions, strips TypeScript, and prefixes identifiers.
/// When `for_caching` is true, simple identifiers are wrapped with safety check.
pub fn generate_event_handler(
    ctx: &mut CodegenContext,
    exp: &ExpressionNode<'_>,
    for_caching: bool,
) {
    match exp {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                ctx.push("\"");
                ctx.push(simple.content);
                ctx.push("\"");
                return;
            }

            let processed: String = if simple.is_ref_transformed {
                simple.content.into()
            } else {
                let content = &simple.content;

                // Step 1: Strip TypeScript if needed
                let ts_stripped: String = if ctx.options.is_ts && content.contains(" as ") {
                    crate::steps::strip_typescript_from_expression(content)
                } else {
                    (*content).into()
                };

                // Step 2: Prefix identifiers if needed. When the checked
                // text is still the node's own bytes, the retained AST
                // applies (P1-7); TS-stripped text that changed falls back.
                if ctx.options.prefix_identifiers {
                    if ts_stripped.as_str() == simple.content {
                        prefix_identifiers_with_context_node(simple, ctx)
                    } else {
                        prefix_identifiers_with_context(&ts_stripped, ctx)
                    }
                } else {
                    ts_stripped
                }
            };
            let processed = if ctx.has_slot_params() && contains_slot_param_scope_prefix(&processed)
            {
                strip_scope_prefixes_for_slot_params(ctx, &processed)
            } else {
                processed
            };

            // Check if it's already an arrow function or function expression.
            // When `processed` is still the node's own bytes the retained AST
            // applies (P1-7); rewritten text keeps the legacy string parse.
            let is_node_text = processed.as_str() == simple.content;
            let is_function = if is_node_text {
                crate::steps::expression::is_function_expression_node(simple)
            } else {
                crate::steps::expression::is_function_expression(&processed)
            };
            if is_function {
                ctx.push(&processed);
                return;
            }

            // Check if it's a simple identifier or member expression (method name/reference)
            let is_member_ref = if is_node_text {
                crate::steps::expression::is_event_handler_reference_node(simple)
            } else {
                is_simple_member_expression(&processed)
            };
            if crate::steps::is_simple_identifier(&processed) || is_member_ref {
                if for_caching {
                    ctx.push("(...args) => (");
                    ctx.push(&processed);
                    ctx.push(" && ");
                    ctx.push(&processed);
                    ctx.push("(...args))");
                } else {
                    ctx.push(&processed);
                }
                return;
            }

            // Compound expression: wrap as arrow function. Vue emits the
            // multi-statement body with no surrounding spaces (`$event => {...}`).
            if processed.contains(';') {
                ctx.push("$event => {");
                ctx.push(&processed);
                ctx.push("}");
            } else {
                ctx.push("$event => (");
                ctx.push(&processed);
                ctx.push(")");
            }
        }
        ExpressionNode::Compound(comp) => {
            for child in comp.children.iter() {
                match child {
                    CompoundExpressionChild::Simple(exp) => {
                        generate_simple_expression(ctx, exp);
                    }
                    CompoundExpressionChild::String(s) => {
                        ctx.push(s);
                    }
                    CompoundExpressionChild::Symbol(helper) => {
                        ctx.push(ctx.helper(*helper));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::generate_simple_expression_with_prefix;
    use crate::codegen::context::CodegenContext;
    use crate::codegen::expression::{generate_event_handler, generate_simple_expression};
    use crate::options::{BindingMetadata, BindingType, CodegenOptions};
    use crate::{ExpressionNode, SimpleExpressionNode, SourceLocation};
    use vize_s0::{Allocator, FxHashMap};

    #[test]
    fn test_shorthand_property_expansion() {
        let mut bindings = FxHashMap::default();
        bindings.insert("foo".into(), BindingType::SetupConst);
        let metadata = BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let options = CodegenOptions {
            inline: false,
            binding_metadata: Some(metadata),
            ..Default::default()
        };

        let ctx = CodegenContext::new(options);
        let result = generate_simple_expression_with_prefix(&ctx, "{ foo }");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_assignment_setup_let_adds_value() {
        let mut bindings = FxHashMap::default();
        bindings.insert("count".into(), BindingType::SetupLet);
        let metadata = BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let options = CodegenOptions {
            inline: false,
            binding_metadata: Some(metadata),
            ..Default::default()
        };

        let ctx = CodegenContext::new(options);
        let result = generate_simple_expression_with_prefix(&ctx, "count = count + 1");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_assignment_setup_ref_adds_value() {
        let mut bindings = FxHashMap::default();
        bindings.insert("quoteId".into(), BindingType::SetupRef);
        let metadata = BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let options = CodegenOptions {
            inline: false,
            binding_metadata: Some(metadata),
            ..Default::default()
        };

        let ctx = CodegenContext::new(options);
        let result = generate_simple_expression_with_prefix(&ctx, "quoteId = null");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_assignment_setup_ref_inline_adds_value() {
        let mut bindings = FxHashMap::default();
        bindings.insert("quoteId".into(), BindingType::SetupRef);
        bindings.insert("renoteTargetNote".into(), BindingType::SetupRef);
        let metadata = BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let options = CodegenOptions {
            inline: true,
            binding_metadata: Some(metadata),
            ..Default::default()
        };

        let ctx = CodegenContext::new(options);
        let result = generate_simple_expression_with_prefix(
            &ctx,
            "quoteId = null; renoteTargetNote = null;",
        );
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_static_string_escaping() {
        let mut ctx = CodegenContext::new(CodegenOptions::default());
        let exp = SimpleExpressionNode::new("Line 1\nLine 2", true, SourceLocation::STUB);
        generate_simple_expression(&mut ctx, &exp);
        let output = ctx.into_code();
        insta::assert_snapshot!(output.as_str());
    }

    #[test]
    fn test_generate_event_handler_keeps_ref_transformed_arrow() {
        let mut bindings = FxHashMap::default();
        bindings.insert("selectedFolders".into(), BindingType::SetupRef);
        bindings.insert("folder".into(), BindingType::SetupRef);
        let metadata = BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let options = CodegenOptions {
            inline: true,
            binding_metadata: Some(metadata),
            prefix_identifiers: true,
            is_ts: true,
            ..Default::default()
        };

        let allocator = Allocator::new();
        let mut ctx = CodegenContext::new(options);
        let exp = ExpressionNode::Simple(vize_s0::Box::new_in(
            SimpleExpressionNode {
                content: "$event => (selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id))",
                is_static: false,
                const_type: crate::ConstantType::NotConstant,
                loc: SourceLocation::STUB,
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: true,
                is_ref_transformed: true,
            },
            &&allocator,
        ));

        generate_event_handler(&mut ctx, &exp, true);
        let output = ctx.into_code();
        assert_eq!(
            output.as_str(),
            "$event => (selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id))"
        );
    }
}
