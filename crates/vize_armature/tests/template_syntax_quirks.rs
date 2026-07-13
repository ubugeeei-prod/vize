use vize_armature::{parse_with_options, parse_with_options_and_template_syntax};
use vize_carton::Bump;
use vize_relief::{
    errors::ErrorCode,
    options::{ParserOptions, TemplateSyntaxMode},
};

#[test]
fn quirks_accepts_adjacent_attributes_recovered_by_vue_compiler() {
    let allocator = Bump::new();
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
    let allocator = Bump::new();
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
