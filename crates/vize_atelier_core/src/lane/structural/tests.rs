// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod structural_transform_tests {
    use vize_s0::Allocator;

    use super::super::super::traverse::traverse_children;
    use super::super::*;
    use crate::errors::CompilerError;
    use crate::lane::{ParentNode, TransformContext};
    use crate::options::TransformOptions;
    use crate::parser::parse;

    fn transform_errors(source: &str) -> std::vec::Vec<CompilerError> {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
        traverse_children(&mut ctx, ParentNode::Root(&mut root as *mut _));
        ctx.errors
    }

    #[test]
    fn test_v_if_without_expression_reports_error() {
        let errors = transform_errors(r#"<div v-if>always</div>"#);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::VIfNoExpression);
    }

    #[test]
    fn test_v_else_if_without_expression_reports_error() {
        let errors = transform_errors(r#"<div v-if="ok">yes</div><div v-else-if>maybe</div>"#);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::VIfNoExpression);
    }

    #[test]
    fn test_v_else_without_expression_stays_valid() {
        let errors = transform_errors(r#"<div v-if="ok">yes</div><div v-else>no</div>"#);

        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
    }
}
