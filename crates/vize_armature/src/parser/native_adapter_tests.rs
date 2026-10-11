//! Parser-level differential proof for the opt-in shared lexer adapter.

#![expect(
    clippy::disallowed_macros,
    reason = "compare complete debug ASTs in the test oracle"
)]

use super::Parser;
use super::compat_reference::parse_with_compat as parse_with_adapter;
use vize_l0::Allocator;
use vize_relief::options::ParserOptions;

fn assert_parser_parity_with_options(source: &str, document: bool, options: ParserOptions) {
    let old_allocator = Allocator::new();
    let new_allocator = Allocator::new();
    let old = if document {
        Parser::document_with_options(&old_allocator, source, options.clone()).parse()
    } else {
        Parser::with_options(&old_allocator, source, options.clone()).parse()
    };
    let new = if document {
        parse_with_adapter(Parser::document_with_options(
            &new_allocator,
            source,
            options,
        ))
    } else {
        parse_with_adapter(Parser::with_options(&new_allocator, source, options))
    };
    assert_eq!(format!("{:#?}", new.0), format!("{:#?}", old.0), "{source}");
    assert_eq!(format!("{:#?}", new.1), format!("{:#?}", old.1), "{source}");
}

fn assert_parser_parity(source: &str, document: bool) {
    assert_parser_parity_with_options(source, document, ParserOptions::default());
}

#[test]
fn shared_lexer_adapter_preserves_parser_ast_and_diagnostics() {
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        assert_parser_parity(fixture.source, false);
    }
    // The original multi-scalar source is retained as a named consumer change;
    // public compatibility events keep their first-scalar law.
    super::attribute_entity_witness::tests::assert_complete_change(
        "native-adapter-fjlig",
        "<p title='&fjlig;'>&fjlig;</p>",
        false,
        ParserOptions::default(),
        &[super::attribute_entity_witness::Change {
            directive: false,
            name: "title",
            before: "f",
            after: "fj",
        }],
    );
    for source in ["<x / >", "<div a='unfinished"] {
        assert_parser_parity(source, false);
    }
    assert_parser_parity("<!DOCTYPE html><main>{{ value }}</main>", true);
    assert_parser_parity_with_options(
        "<p>[[ value ]] and {{ literal }}</p>",
        false,
        ParserOptions {
            delimiters: ("[[".into(), "]]".into()),
            ..ParserOptions::default()
        },
    );
    assert_parser_parity_with_options(
        "<LegacySelect\n  :options=\"options\"\n  // @vue-expect-error legacy API\n  :selected-id=\"selectedId\"\n/>",
        false,
        ParserOptions {
            experimental_in_tag_comments: true,
            ..ParserOptions::default()
        },
    );
    #[cfg(feature = "legacy")]
    assert_parser_parity_with_options(
        "<p>{{{ rawHtml }}} and {{ text }}</p>",
        false,
        ParserOptions {
            dialect: vize_l0::config::VueVersion::V1,
            ..ParserOptions::default()
        },
    );
}
