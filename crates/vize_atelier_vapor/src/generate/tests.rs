#![expect(
    clippy::disallowed_macros,
    reason = "test fixtures and insta snapshots use std strings and format"
)]
// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
mod vapor_generate_tests {
    use super::super::{generate_vapor, spans::TEMPLATE_ESCAPES};
    use crate::lower::transform_to_ir;
    use vize_atelier_core::codegen::document::EmitDocument;
    use vize_atelier_core::parser::parse;
    use vize_carton::Allocator;

    #[test]
    fn test_generate_simple() {
        let allocator = Allocator::new();
        let source = "<div>hello</div>";
        let (root, _) = parse(&allocator, source);
        let ir = transform_to_ir(&allocator, &root, source);
        let result = generate_vapor(&ir, None);

        assert!(!result.code.is_empty());
        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_generate_with_event() {
        let allocator = Allocator::new();
        let source = r#"<button @click="handleClick">Click</button>"#;
        let (root, _) = parse(&allocator, source);
        let ir = transform_to_ir(&allocator, &root, source);
        let result = generate_vapor(&ir, None);

        insta::assert_snapshot!(result.code.as_str());
    }

    /// Template strings are escaped through the emission document.
    fn escape_template(s: &str) -> vize_carton::String {
        let mut out = EmitDocument::new(false);
        out.push_escaped(&EmitDocument::plain(s), &TEMPLATE_ESCAPES);
        out.into_string()
    }

    #[test]
    fn test_escape_template() {
        assert_eq!(escape_template("hello"), "hello");
        assert_eq!(escape_template("a\\b\r"), "a\\\\b\\r");
        assert_eq!(escape_template("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_template("hello\"world"), "hello\\\"world");
    }
}
