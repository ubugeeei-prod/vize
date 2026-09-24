//! Parser behaviour under the opt-in legacy Vue dialects.

use super::parse_with_options;
use vize_relief::{ExpressionNode, TemplateChildNode, options::ParserOptions};
use vize_s0::Allocator;

#[test]
fn triple_mustache_under_v1_lowers_to_raw_html_interpolation() {
    use vize_s0::config::VueVersion;

    let allocator = Allocator::new();
    let options = ParserOptions {
        dialect: VueVersion::V1,
        ..ParserOptions::default()
    };
    let (root, errors) = parse_with_options(&allocator, "{{{ rawHtml }}}", options);

    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(root.children.len(), 1, "single raw-HTML interpolation");

    let TemplateChildNode::Interpolation(interp) = &root.children[0] else {
        panic!("expected interpolation node");
    };
    assert!(
        interp.raw,
        "Vue 1.x `{{{{{{ … }}}}}}` is a raw-HTML interpolation"
    );
    let ExpressionNode::Simple(expr) = &interp.content else {
        panic!("expected simple expression");
    };
    // The extra braces are stripped from the expression and the node spans the
    // full triple-mustache.
    assert_eq!(expr.content, "rawHtml");
    assert_eq!(interp.loc.span.slice("{{{ rawHtml }}}"), "{{{ rawHtml }}}");
}

#[test]
fn v1_double_mustache_stays_escaped_alongside_triple() {
    use vize_s0::config::VueVersion;

    let allocator = Allocator::new();
    let options = ParserOptions {
        dialect: VueVersion::V1,
        ..ParserOptions::default()
    };
    let (root, errors) = parse_with_options(&allocator, "{{ a }} {{{ b }}}", options);

    assert!(errors.is_empty(), "{errors:?}");
    let interps: std::vec::Vec<_> = root
        .children
        .iter()
        .filter_map(|c| match c {
            TemplateChildNode::Interpolation(i) => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(interps.len(), 2);
    // Plain `{{ a }}` is escaped; `{{{ b }}}` is raw.
    assert!(!interps[0].raw);
    assert!(interps[1].raw);
}
