// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
mod ssr_codegen_tests {
    use super::super::SsrCodegenResult;
    use super::super::helpers::{escape_html, escape_html_attr};

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_escape_html_all_special_chars() {
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("<"), "&lt;");
        assert_eq!(escape_html(">"), "&gt;");
        assert_eq!(escape_html("\""), "&quot;");
        assert_eq!(escape_html("'"), "&#39;");
    }

    #[test]
    fn test_escape_html_no_special() {
        assert_eq!(escape_html("hello world"), "hello world");
        assert_eq!(escape_html("abc123"), "abc123");
    }

    #[test]
    fn test_escape_html_attr() {
        assert_eq!(escape_html_attr("hello\"world"), "hello&quot;world");
        assert_eq!(escape_html_attr("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_html_attr_preserves_angle_brackets() {
        // In attribute context, < and > do not need escaping
        assert_eq!(escape_html_attr("<foo>"), "<foo>");
        assert_eq!(escape_html_attr("a > b"), "a > b");
    }

    #[test]
    fn test_escape_html_attr_no_special() {
        assert_eq!(escape_html_attr("hello"), "hello");
    }

    #[test]
    fn test_ssr_codegen_result_default() {
        let result = SsrCodegenResult::default();
        assert!(result.code.is_empty());
        assert!(result.preamble.is_empty());
    }
}
