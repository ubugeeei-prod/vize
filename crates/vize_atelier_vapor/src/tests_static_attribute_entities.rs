//! Legacy Vapor HTML attribute serialization regressions from #7502.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures use assertions, std strings and format"
)]

use super::compile_vapor;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn assert_parses_as_module(code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        code,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    )
    .parse();

    assert!(
        parsed.diagnostics.is_empty(),
        "generated code should parse, got: {:?}\n\n{}",
        parsed.diagnostics,
        code
    );
}

#[test]
fn test_compile_static_attribute_entities_reported_in_issue_7502() {
    let allocator = vize_carton::Allocator::new();
    let source = include_str!("../tests/fixtures/static-attribute-double-decode.input.txt").trim();
    let result = compile_vapor(&allocator, source, Default::default());
    let expected = r#"<div title="a &amp;lt; b">x</div>"#;

    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    assert_eq!(result.templates.len(), 1);
    assert_eq!(result.templates[0], expected);
    assert!(
        result
            .code
            .contains(&serde_json::to_string(expected).unwrap())
    );
    assert_parses_as_module(&result.code);
}

#[test]
fn test_compile_static_attribute_entities_preserve_html_attribute_boundaries() {
    for (source, expected) in [
        (
            r#"<div title="&amp;lt;">x</div>"#,
            r#"<div title="&amp;lt;">x</div>"#,
        ),
        (
            r#"<div title="&#38;copy;">x</div>"#,
            r#"<div title="&amp;copy;">x</div>"#,
        ),
        (
            r#"<div title="&#x26;#60;">x</div>"#,
            r#"<div title="&amp;#60;">x</div>"#,
        ),
        (
            r#"<div title="&amp;amp;lt;">x</div>"#,
            r#"<div title="&amp;amp;lt;">x</div>"#,
        ),
        (r#"<div title="a&b">x</div>"#, r#"<div title="a&b">x</div>"#),
        (
            r#"<div title='a&amp;lt;"b'>x</div>"#,
            r#"<div title="a&amp;lt;&quot;b">x</div>"#,
        ),
        (
            r#"<div title='a"b'>x</div>"#,
            r#"<div title="a&quot;b">x</div>"#,
        ),
        (
            r#"<div title=a&b>x</div>"#,
            r#"<div title="a&amp;b">x</div>"#,
        ),
        (
            r#"<div title="&lt;b&gt;">x</div>"#,
            r#"<div title="&lt;b&gt;">x</div>"#,
        ),
        (
            r#"<div title=&amp;lt;>x</div>"#,
            r#"<div title="&amp;lt;">x</div>"#,
        ),
        (
            r#"<div class="a&amp;amp;b">x</div>"#,
            r#"<div class="a&amp;amp;b">x</div>"#,
        ),
        (
            r#"<div title="a &amp;lt; b">&amp;lt;</div>"#,
            r#"<div title="a &amp;lt; b">&amp;lt;</div>"#,
        ),
        (
            r#"<div title="plain">x</div>"#,
            r#"<div title="plain">x</div>"#,
        ),
    ] {
        let allocator = vize_carton::Allocator::new();
        let result = compile_vapor(&allocator, source, Default::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        assert_eq!(result.templates.len(), 1, "{source}");
        assert_eq!(result.templates[0], expected, "{source}");
        assert!(
            result
                .code
                .contains(&serde_json::to_string(expected).unwrap()),
            "{source}"
        );
        assert_parses_as_module(&result.code);
    }
}
