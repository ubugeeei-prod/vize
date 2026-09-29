use super::{
    Allocator, ErrorCode, ExpressionNode, ParserOptions, TemplateChildNode, find_element,
    parse_document, parse_document_with_options, parse_with_options,
};

/// Real parse errors (e.g. an unclosed element) are still surfaced in document
/// mode — toleration is scoped to the doctype declaration only.
#[test]
fn test_document_mode_reports_real_errors() {
    let allocator = Allocator::new();
    let src = "<!DOCTYPE html><html><body><div></body></html>";
    let (_root, errors) = parse_document(&allocator, src);
    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingEndTag || e.code == ErrorCode::InvalidEndTag),
        "unclosed <div> should still produce an error: {errors:?}"
    );
}

/// `parse_document_with_options` honors custom parser options (here custom
/// interpolation delimiters) while still tolerating the doctype.
#[test]
fn test_document_mode_with_options() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        delimiters: ("[[".into(), "]]".into()),
        ..Default::default()
    };
    let src = "<!DOCTYPE html><html><body><span>[[ msg ]]</span></body></html>";
    let (root, errors) = parse_document_with_options(&allocator, src, options);
    assert!(
        !errors
            .iter()
            .any(|e| e.code == ErrorCode::IncorrectlyOpenedComment),
        "doctype tolerated with options: {errors:?}"
    );
    let span = find_element(&root.children, "span").expect("span present");
    assert!(
        span.children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::Interpolation(_))),
        "custom delimiters should produce an interpolation node"
    );
}

/// Vue 2/3 treat `{{{ x }}}` as a `{{ }}` mustache plus trailing `}` text.
#[test]
fn triple_mustache_is_a_braced_mustache_outside_legacy_v1() {
    use vize_l0::config::VueVersion;

    for dialect in [VueVersion::V3, VueVersion::V2] {
        let allocator = Allocator::new();
        let options = ParserOptions {
            dialect,
            ..Default::default()
        };
        let (root, errors) = parse_with_options(&allocator, "{{{ rawHtml }}}", options);

        assert!(errors.is_empty(), "{dialect:?}: {errors:?}");
        assert_eq!(
            root.children.len(),
            2,
            "{dialect:?}: interp + trailing text"
        );

        match &root.children[0] {
            TemplateChildNode::Interpolation(interp) => {
                let ExpressionNode::Simple(expr) = &interp.content else {
                    panic!("expected simple expression");
                };
                // The leading brace stays inside the expression, exactly as today.
                assert_eq!(expr.content, "{ rawHtml");
            }
            other => panic!(
                "{dialect:?}: expected interpolation, got {:?}",
                other.node_type()
            ),
        }
        match &root.children[1] {
            TemplateChildNode::Text(text) => assert_eq!(text.content, "}"),
            other => panic!(
                "{dialect:?}: expected trailing text, got {:?}",
                other.node_type()
            ),
        }
    }
}
