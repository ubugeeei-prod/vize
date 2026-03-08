//! Event handler and expression generation.
//!
//! Handles generating event handler expressions with proper arrow function
//! wrapping, TypeScript stripping, and identifier prefixing.

use crate::ast::{CompoundExpressionChild, ExpressionNode};

use super::{
    super::context::CodegenContext,
    generate_simple_expression,
    helpers::{prefix_identifiers_with_context, strip_ctx_for_slot_params},
};
use vize_carton::String;

/// Generate a simple expression with appropriate prefix.
/// Used for ref attribute values that need `$setup.` prefix in function mode.
#[allow(dead_code)]
pub fn generate_simple_expression_with_prefix(ctx: &CodegenContext, content: &str) -> String {
    prefix_identifiers_with_context(content, ctx)
}

/// Check if a string is a simple member expression like `_ctx.foo` or `$setup.bar`.
/// This is used to determine if an event handler needs wrapping.
pub fn is_simple_member_expression(s: &str) -> bool {
    if let Some(dot_pos) = s.find('.') {
        let prefix = &s[..dot_pos];
        let suffix = &s[dot_pos + 1..];
        let valid_prefix = prefix == "_ctx" || prefix == "$setup" || prefix == "$props";
        let valid_suffix = !suffix.is_empty()
            && !suffix.contains('.')
            && !suffix.contains('(')
            && !suffix.contains('[');
        return valid_prefix && valid_suffix;
    }
    false
}

/// Check if an event handler expression is an inline handler.
/// Inline handlers are expressions that are NOT simple identifiers or member expressions.
#[allow(dead_code)]
pub fn is_inline_handler(exp: &ExpressionNode<'_>) -> bool {
    match exp {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                return false;
            }

            let content = simple.loc.source.as_str();

            if content.contains("=>") || content.trim().starts_with("function") {
                return false;
            }

            if crate::transforms::is_simple_identifier(content)
                || is_simple_member_expression(content)
            {
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
                ctx.push(&simple.content);
                ctx.push("\"");
                return;
            }

            let content = &simple.content;

            // Step 1: Strip TypeScript if needed
            let ts_stripped: String = if ctx.options.is_ts && content.contains(" as ") {
                crate::transforms::strip_typescript_from_expression(content)
            } else {
                content.clone()
            };

            // Step 2: Prefix identifiers if needed
            let processed: String = if ctx.options.prefix_identifiers {
                prefix_identifiers_with_context(&ts_stripped, ctx)
            } else {
                ts_stripped
            };
            let processed = if ctx.has_slot_params() && processed.contains("_ctx.") {
                strip_ctx_for_slot_params(ctx, &processed)
            } else {
                processed
            };

            // Check if it's already an arrow function or function expression
            if processed.contains("=>") || processed.trim().starts_with("function") {
                ctx.push(&processed);
                return;
            }

            // Check if it's a simple identifier or member expression (method name/reference)
            if crate::transforms::is_simple_identifier(&processed)
                || is_simple_member_expression(&processed)
            {
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

            // Compound expression: wrap as arrow function
            if processed.contains(';') {
                ctx.push("$event => { ");
                ctx.push(&processed);
                ctx.push(" }");
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
mod tests {
    use super::generate_simple_expression_with_prefix;
    use crate::ast::{SimpleExpressionNode, SourceLocation};
    use crate::codegen::context::CodegenContext;
    use crate::codegen::expression::generate_simple_expression;
    use crate::options::{BindingMetadata, BindingType, CodegenOptions};
    use vize_carton::FxHashMap;

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
        assert!(result.contains("foo: $setup.foo"), "Got: {}", result);
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
        assert!(result.contains("count.value"), "Got: {}", result);
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
        assert!(
            result.contains("quoteId.value"),
            "SetupRef assignment should add .value. Got: {}",
            result
        );
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
        assert!(
            result.contains("quoteId.value"),
            "SetupRef assignment in inline mode should add .value. Got: {}",
            result
        );
        assert!(
            result.contains("renoteTargetNote.value"),
            "SetupRef assignment in inline mode should add .value. Got: {}",
            result
        );
    }

    #[test]
    fn test_static_string_escaping() {
        let mut ctx = CodegenContext::new(CodegenOptions::default());
        let exp = SimpleExpressionNode::new("Line 1\nLine 2", true, SourceLocation::STUB);
        generate_simple_expression(&mut ctx, &exp);
        let output = ctx.into_code();
        assert!(
            output.contains("\\n"),
            "Expected newline to be escaped. Got: {}",
            output
        );
        assert!(
            !output.contains("Line 1\nLine 2"),
            "Expected raw newline to be escaped. Got: {}",
            output
        );
    }
}
