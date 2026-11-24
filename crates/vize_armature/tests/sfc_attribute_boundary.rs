use vize_armature::parse_with_options_and_template_syntax;
use vize_l0::{Allocator, cstr};
use vize_relief::options::{ParseMode, ParserOptions, TemplateSyntaxMode};

#[test]
fn sfc_boundaries_preserve_the_complete_standard_tree_without_its_html_warning() {
    let allocator = Allocator::new();
    let source = r#"<div id="a"class="b"></div>"#;
    for syntax in [TemplateSyntaxMode::Standard, TemplateSyntaxMode::Strict] {
        let (standard, errors) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions::default(),
            syntax,
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].code,
            vize_l0::ErrorCode::MissingWhitespaceBetweenAttributes
        );
        let (sfc, sfc_errors) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions {
                mode: ParseMode::Sfc,
                ..ParserOptions::default()
            },
            syntax,
        );
        assert!(sfc_errors.is_empty(), "{sfc_errors:?}");
        assert_eq!(cstr!("{sfc:?}"), cstr!("{standard:?}"));
    }
}

#[test]
fn sfc_format_does_not_reclassify_genuine_parse_errors() {
    let allocator = Allocator::new();
    for (source, expected) in [
        (
            r#"<div id="x"=oops></div>"#,
            vize_l0::ErrorCode::UnexpectedEqualsSignBeforeAttributeName,
        ),
        ("<div", vize_l0::ErrorCode::EofInTag),
    ] {
        let (standard, errors) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions::default(),
            TemplateSyntaxMode::Standard,
        );
        assert!(
            errors.iter().any(|error| error.code == expected),
            "{errors:?}"
        );
        let (sfc, sfc_errors) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions {
                mode: ParseMode::Sfc,
                ..ParserOptions::default()
            },
            TemplateSyntaxMode::Standard,
        );
        assert_eq!(cstr!("{sfc_errors:?}"), cstr!("{errors:?}"));
        assert_eq!(cstr!("{sfc:?}"), cstr!("{standard:?}"));
    }
}
