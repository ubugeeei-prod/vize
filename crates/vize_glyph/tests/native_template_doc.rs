//! Genuine native L1 parsing and source-preserving template layout laws.

use vize_glyph::native_doc::{
    LineEnding, PrintOptions, TemplateRefusal, UnsupportedSyntax, print, template_document,
};
use vize_l0::{Allocator, String};
use vize_l1::dialect::vue3::surface::parse_component;

fn format(source: &str, width: usize) -> String {
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, source).unwrap();
    let document = template_document(&parsed, &allocator).unwrap();
    assert!(core::ptr::eq(document.original(), &parsed));
    assert_eq!(vize_l1::check_fidelity(&parsed.tree), Ok(()));
    print(
        document.document(),
        &PrintOptions {
            width,
            ..PrintOptions::default()
        },
    )
}

#[test]
fn native_attributes_flatten_or_break_with_same_order_quotes_and_entities() {
    let source = "<Card z = '日本&amp;'  disabled  id=main />";
    assert_eq!(
        format(source, 80),
        "<Card z='日本&amp;' disabled id=main />"
    );
    assert_eq!(
        format(source, 25),
        "<Card\n  z='日本&amp;'\n  disabled\n  id=main\n/>"
    );
}

#[test]
fn content_and_comment_whitespace_remain_byte_exact() {
    let source = "<!-- before\r\n exact -->\r\n<p class = 'red' title=\"a  b\"> A\t&amp; B \r\n<!--c--> C </p>\r\n";
    assert_eq!(
        format(source, 80),
        "<!-- before\r\n exact -->\r\n<p class='red' title=\"a  b\"> A\t&amp; B \r\n<!--c--> C </p>\r\n"
    );
}

#[test]
fn nested_opening_wrap_uses_real_structural_depth() {
    let source = "<div>\n  <Card first=\"12345\" second=\"67890\"/>\n</div>";
    assert_eq!(
        format(source, 20),
        "<div>\n  <Card\n    first=\"12345\"\n    second=\"67890\"\n  />\n</div>"
    );
}

#[test]
fn multiline_attribute_values_are_preserved_without_normalization() {
    assert_eq!(
        format("<p title='one\r\n two' id = x></p>", 80),
        "<p\n  title='one\r\n two'\n  id=x\n></p>"
    );
}

#[test]
fn generated_crlf_keeps_source_content_newlines() {
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, "<p a=x b=y>\ncontent\n</p>").unwrap();
    let doc = template_document(&parsed, &allocator).unwrap();
    assert_eq!(
        print(
            doc.document(),
            &PrintOptions {
                width: 6,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "<p\r\n  a=x\r\n  b=y\r\n>\ncontent\n</p>"
    );
}

#[test]
fn native_layout_is_a_fixed_point_for_supported_templates() {
    for source in [
        "<Card z = 'é&amp;' id=x />",
        "<p a=x b=y> text  </p>",
        "<!-- exact -->\n<div><input a=x b=y></div>",
    ] {
        for width in [0, 7, 20, 80] {
            let output = format(source, width);
            assert_eq!(format(&output, width), output);
        }
    }
}

#[test]
fn recovered_syntax_keeps_original_errors_and_bytes_on_refusal() {
    for source in [
        "<p a='unterminated",
        "<p>",
        "</stray>",
        "<p a= >x</p>",
        "<Card id=x/>",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, source).unwrap();
        let count = parsed.errors.len();
        assert!(matches!(
            template_document(&parsed, &allocator),
            Err(TemplateRefusal::Recovered { .. })
        ));
        assert_eq!(parsed.errors.len(), count);
        assert_eq!(vize_l1::check_fidelity(&parsed.tree), Ok(()));
        assert_eq!(parsed.tree.source, source);
    }
}

#[test]
fn incomplete_directives_use_shared_l1_syntax_and_are_explicitly_unsupported() {
    for source in [
        "<p :[].camel='value'/>",
        "<p :='value'/>",
        "<p @[]='act()'/>",
        "<p #[]='item'/>",
        "<p v-/>",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, source).unwrap();
        assert!(matches!(
            template_document(&parsed, &allocator),
            Err(TemplateRefusal::Unsupported {
                syntax: UnsupportedSyntax::Directive,
                ..
            })
        ));
        assert_eq!(vize_l1::check_fidelity(&parsed.tree), Ok(()));
    }
}

#[test]
fn embedded_expression_is_not_reparsed_or_silently_admitted() {
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, "<p>{{ user?.name }}</p>").unwrap();
    assert!(matches!(
        template_document(&parsed, &allocator),
        Err(TemplateRefusal::Unsupported {
            syntax: UnsupportedSyntax::Interpolation,
            ..
        })
    ));
    assert_eq!(vize_l1::check_fidelity(&parsed.tree), Ok(()));
}

#[test]
fn equal_bytes_from_foreign_source_do_not_establish_source_custody() {
    let allocator = Allocator::default();
    let text = String::from("<p a=x/>");
    let foreign = text.clone();
    let mut parsed = parse_component(&allocator, &text).unwrap();
    parsed.tree.source = &foreign;
    assert!(matches!(
        template_document(&parsed, &allocator),
        Err(TemplateRefusal::SourceMismatch { offset: 0 })
    ));
}
