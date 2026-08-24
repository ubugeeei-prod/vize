// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
mod transform_expression_tests {
    use super::super::{
        MAX_EXPRESSION_NESTING_DEPTH, clone_expression, expression_exceeds_max_depth,
        expression_nesting_depth, is_event_handler_reference_expression, is_function_expression,
        prefix::prefix_identifiers_in_expression, process_expression,
        typescript::strip_typescript_from_expression,
    };
    use crate::{
        CompoundExpressionNode, ExpressionNode, RuntimeHelper, SourceLocation,
        lane::TransformContext,
        options::{BindingMetadata, BindingType, TransformOptions},
    };
    use vize_carton::{Allocator, Box, FxHashMap};

    fn test_context<'a>(allocator: &'a Allocator, source: &'a str) -> TransformContext<'a> {
        let mut bindings = FxHashMap::default();
        bindings.insert("selectedFolders".into(), BindingType::SetupRef);
        bindings.insert("folder".into(), BindingType::SetupRef);

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
    fn test_process_expression_rewrites_compound_ts_ref_reads() {
        let allocator = Allocator::new();
        let source = "!selectedFolders.some(f => f.id === folder!.id)";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert!(result.content.starts_with("!selectedFolders.value.some("));
        assert!(result.content.contains("folder.value.id"));
    }

    #[test]
    fn test_process_expression_uses_setup_proxy_in_function_mode() {
        let allocator = Allocator::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("isExternal".into(), BindingType::SetupRef);

        let source = "isExternal && isExternal.value";
        let mut ctx = TransformContext::new(
            &allocator,
            source,
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
        let expr = compound_expression(&allocator, source);

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert_eq!(
            result.content,
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
    fn test_process_expression_reports_invalid_expression() {
        let allocator = Allocator::new();
        let source = "foo(";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        // Raw passthrough (matches vue-core, which returns the node
        // unchanged), but with a compile diagnostic instead of silence.
        assert_eq!(result.content, "foo(");
        assert_eq!(ctx.errors.len(), 1, "errors: {:?}", ctx.errors);
        assert_eq!(
            ctx.errors[0].code,
            crate::errors::ErrorCode::InvalidExpression
        );
        assert!(
            ctx.errors[0]
                .message
                .starts_with("Error parsing JavaScript expression: "),
            "message: {:?}",
            ctx.errors[0].message
        );
        assert!(ctx.errors[0].loc.is_some(), "diagnostic must carry a span");
    }

    #[test]
    fn test_process_expression_keyword_identifier_has_no_diagnostic() {
        // `class` fails to parse as an expression but is a rewritable simple
        // identifier; vue-core never parses it (simple-identifier fast path)
        // and emits no error.
        let allocator = Allocator::new();
        let source = "class";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_expression(&mut ctx, &expr, false);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert_eq!(result.content, "_ctx.class");
        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);
    }

    #[test]
    fn test_process_expression_valid_ts_fallback_has_no_diagnostic() {
        // `foo<string>` is a TS instantiation expression. The TS-stripping
        // heuristic does not lower it, so the JS parse fails — but the
        // official compiler (babel + typescript plugin) accepts it, and the
        // parity rule forbids rejecting what the official compiler accepts.
        let allocator = Allocator::new();
        let source = "foo<string>";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let _ = process_expression(&mut ctx, &expr, false);
        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);
    }

    #[test]
    fn test_process_expression_ts_slot_params_have_no_diagnostic() {
        let allocator = Allocator::new();
        let source = "{ open, close }: { open: boolean, close?: () => void }";
        let mut ctx = test_context(&allocator, source);
        let expr = compound_expression(&allocator, source);

        let result = process_expression(&mut ctx, &expr, true);
        let ExpressionNode::Simple(result) = result else {
            panic!("expected simple expression");
        };

        assert_eq!(result.content, source);
        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);
    }

    #[test]
    fn test_clone_expression_preserves_compound_source() {
        let allocator = Allocator::new();
        let source = "foo + bar";
        let expr = compound_expression(&allocator, source);

        let cloned = clone_expression(&expr, &allocator, source);
        let ExpressionNode::Simple(simple) = cloned else {
            panic!("expected clone_expression to flatten Compound to Simple");
        };

        assert_eq!(simple.content, source);
    }
}
