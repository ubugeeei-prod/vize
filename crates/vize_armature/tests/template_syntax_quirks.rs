use vize_armature::{parse_with_options, parse_with_options_and_template_syntax};
use vize_relief::{
    errors::ErrorCode,
    options::{ParserOptions, TemplateSyntaxMode},
};
use vize_s0::Allocator;

#[test]
fn quirks_accepts_adjacent_attributes_recovered_by_vue_compiler() {
    let allocator = Allocator::new();
    let source = r#"<div id="a"class="b"></div>"#;
    let (_, standard_errors) = parse_with_options(&allocator, source, ParserOptions::default());
    let (_, quirks_errors) = parse_with_options_and_template_syntax(
        &allocator,
        source,
        ParserOptions::default(),
        TemplateSyntaxMode::Quirks,
    );

    assert!(
        standard_errors
            .iter()
            .any(|error| error.code == ErrorCode::MissingWhitespaceBetweenAttributes)
    );
    assert!(quirks_errors.is_empty(), "{quirks_errors:?}");
}

#[test]
fn quirks_ignores_closing_tags_for_void_elements() {
    let allocator = Allocator::new();
    let source = "<img></img>";
    let (_, standard_errors) = parse_with_options(&allocator, source, ParserOptions::default());
    let (_, quirks_errors) = parse_with_options_and_template_syntax(
        &allocator,
        source,
        ParserOptions::default(),
        TemplateSyntaxMode::Quirks,
    );

    assert!(
        standard_errors
            .iter()
            .any(|error| error.code == ErrorCode::InvalidEndTag)
    );
    assert!(quirks_errors.is_empty(), "{quirks_errors:?}");
}

#[test]
fn quirks_preserves_vue_tree_shape_for_html_recovery_cases() {
    let cases = [
        (
            "nested button",
            "<button><span><button>share</button></span></button>",
        ),
        (
            "nested anchor",
            "<a href=\"#outer\"><span><a href=\"#inner\">open</a></span></a>",
        ),
        (
            "nested form",
            "<form><section><form><input></form></section></form>",
        ),
        (
            "table foster parenting",
            "<table><div>description</div><tr><td>value</td></tr></table>",
        ),
    ];

    for (name, source) in cases {
        let allocator = Allocator::new();
        let (_, standard_errors) = parse_with_options(&allocator, source, ParserOptions::default());
        assert!(
            !standard_errors.is_empty(),
            "{name} should exercise HTML recovery"
        );

        let (root, quirks_errors) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions::default(),
            TemplateSyntaxMode::Quirks,
        );
        assert!(quirks_errors.is_empty(), "{name}: {quirks_errors:?}");
        assert_eq!(
            root.children.len(),
            1,
            "{name} should preserve its lexical root"
        );
    }
}
