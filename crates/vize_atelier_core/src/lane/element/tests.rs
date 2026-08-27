// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod element_transform_tests {
    use vize_s0::Allocator;

    use super::super::transform_element;
    use crate::{
        PropNode, TemplateChildNode,
        errors::{CompilerError, ErrorCode},
        lane::{ParentNode, TransformContext, traverse::traverse_children},
        options::TransformOptions,
        parser::parse,
    };

    fn transform_errors(source: &str) -> std::vec::Vec<CompilerError> {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        traverse_children(&mut ctx, ParentNode::Root(&mut root as *mut _));
        ctx.errors
    }

    fn assert_no_model_update_handler(props: &[PropNode<'_>]) {
        assert!(!props.iter().any(|prop| matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "on"
                    && matches!(
                        &dir.arg,
                        Some(crate::ExpressionNode::Simple(arg))
                            if arg.content == "update:modelValue"
                    )
        )));
    }

    #[test]
    fn test_transform_v_model_without_expression_reports_error() {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, r#"<input v-model />"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        match &mut root.children[0] {
            TemplateChildNode::Element(el) => {
                transform_element(&mut ctx, el);

                assert!(!el.props.iter().any(|prop| matches!(
                    prop,
                    PropNode::Directive(dir) if dir.name == "model"
                )));
            }
            other => panic!(
                "Expected ElementNode, got {:?}",
                std::mem::discriminant(other)
            ),
        }

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelNoExpression);
    }

    #[test]
    fn test_transform_component_v_model_without_expression_reports_error() {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, r#"<MyComponent v-model />"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        match &mut root.children[0] {
            TemplateChildNode::Element(el) => {
                transform_element(&mut ctx, el);

                assert!(!el.props.iter().any(|prop| matches!(
                    prop,
                    PropNode::Directive(dir) if dir.name == "model"
                )));
            }
            other => panic!(
                "Expected ElementNode, got {:?}",
                std::mem::discriminant(other)
            ),
        }

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelNoExpression);
    }

    #[test]
    fn test_transform_v_model_on_v_for_scope_reports_error() {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(
            &allocator,
            r#"<div v-for="item in items"><input v-model="item" /></div>"#,
        );
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        traverse_children(&mut ctx, ParentNode::Root(&mut root as *mut _));

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelOnScope);

        match &root.children[0] {
            TemplateChildNode::For(for_node) => match &for_node.children[0] {
                TemplateChildNode::Element(el) => match &el.children[0] {
                    TemplateChildNode::Element(input) => {
                        assert!(!input.props.iter().any(|prop| matches!(
                            prop,
                            PropNode::Directive(dir) if dir.name == "model"
                        )));
                        assert_no_model_update_handler(input.props.as_slice());
                    }
                    other => panic!("Expected input element, got {:?}", other.node_type()),
                },
                other => panic!("Expected v-for child element, got {:?}", other.node_type()),
            },
            other => panic!("Expected ForNode, got {:?}", other.node_type()),
        }
    }

    #[test]
    fn test_transform_v_model_on_v_slot_scope_reports_error() {
        let errors = transform_errors(
            r#"<MyComponent v-slot="{ item }"><input v-model="item" /></MyComponent>"#,
        );

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::VModelOnScope);
    }

    #[test]
    fn test_transform_v_model_on_scope_property_stays_valid() {
        let errors =
            transform_errors(r#"<div v-for="item in items"><input v-model="item.value" /></div>"#);

        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
    }

    fn assert_v_model_arg_on_element_rejected(source: &str) {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        match &mut root.children[0] {
            TemplateChildNode::Element(el) => {
                transform_element(&mut ctx, el);

                // No v-model binding is generated: the directive itself is
                // dropped and no `onUpdate:*` handler is emitted.
                assert!(
                    !el.props.iter().any(|prop| matches!(
                        prop,
                        PropNode::Directive(dir) if dir.name == "model"
                    )),
                    "v-model directive should be removed for {source}"
                );
                assert!(
                    !el.props.iter().any(|prop| matches!(
                        prop,
                        PropNode::Directive(dir) if dir.name == "on"
                    )),
                    "no update handler should be emitted for {source}"
                );
            }
            other => panic!(
                "Expected ElementNode, got {:?}",
                std::mem::discriminant(other)
            ),
        }

        assert_eq!(ctx.errors.len(), 1, "expected one error for {source}");
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelArgOnElement);
    }

    #[test]
    fn test_transform_v_model_static_arg_on_element_reports_error() {
        // Issue #1169: v-model with an argument is component-only; on a plain
        // element it must be a hard error with no binding generated.
        assert_v_model_arg_on_element_rejected(r#"<input v-model:foo="bar" />"#);
    }

    #[test]
    fn test_transform_v_model_dynamic_arg_on_element_reports_error() {
        // Issue #1169: a dynamic arg on a plain element is rejected too, so the
        // three competing update mechanisms never fire.
        assert_v_model_arg_on_element_rejected(r#"<input v-model:[dynKey]="value" />"#);
    }
}
