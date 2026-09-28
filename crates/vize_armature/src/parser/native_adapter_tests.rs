//! Parser-level differential proof for the opt-in shared lexer adapter.

#![expect(
    clippy::disallowed_macros,
    reason = "compare complete debug ASTs in the test oracle"
)]

use super::{Parser, callbacks::ParserCallbacks, current_whitespace_strategy};
use vize_l0::{Allocator, Vec};
use vize_l1::markup::lex::compat::adapter::CompatSink;
use vize_l1::markup::{Component, Delimiters, Document, LexOptions, Lexer};
use vize_relief::{
    RootNode,
    errors::CompilerError,
    options::{ParserOptions, WhitespaceStrategy},
};

fn parse_with_adapter<'a>(mut parser: Parser<'a>) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
    let allocator = parser.allocator;
    parser.root = Some(RootNode::new(allocator, parser.source));

    let delimiter_open: Vec<'a, u8> =
        Vec::from_iter_in(parser.options.delimiters.0.bytes(), &allocator);
    let delimiter_close: Vec<'a, u8> =
        Vec::from_iter_in(parser.options.delimiters.1.bytes(), &allocator);
    assert!(!delimiter_open.is_empty() && !delimiter_close.is_empty());
    let source = parser.source;
    let document = parser.document;
    let options = LexOptions {
        delimiters: Delimiters {
            open: &delimiter_open,
            close: &delimiter_close,
        },
        in_tag_comments: parser.options.experimental_in_tag_comments,
        raw_interpolation: {
            #[cfg(feature = "legacy")]
            {
                crate::legacy::LegacyDialectCapabilities::for_dialect(parser.options.dialect)
                    .raw_html_interpolation
            }
            #[cfg(not(feature = "legacy"))]
            {
                false
            }
        },
    };
    if document {
        Lexer::<Document, _>::new(
            source,
            CompatSink::new(ParserCallbacks {
                parser: &mut parser,
            }),
            options,
        )
        .run();
    } else {
        Lexer::<Component, _>::new(
            source,
            CompatSink::new(ParserCallbacks {
                parser: &mut parser,
            }),
            options,
        )
        .run();
    }
    parser.flush_pending_text();
    parser.handle_unclosed_elements();
    if let Some(ref mut root) = parser.root {
        match current_whitespace_strategy(parser.options.whitespace) {
            WhitespaceStrategy::Condense => {
                super::condense_whitespace(
                    allocator,
                    &mut root.children,
                    parser.options.is_pre_tag,
                );
            }
            WhitespaceStrategy::Preserve => {
                super::preserve_whitespace(
                    allocator,
                    &mut root.children,
                    parser.options.is_pre_tag,
                );
            }
        }
    }
    parser.into_result()
}

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
    for source in [
        "<p title='&fjlig;'>&fjlig;</p>",
        "<x / >",
        "<div a='unfinished",
    ] {
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

#[test]
fn native_v_pre_mode_preserves_parser_ast_and_diagnostics() {
    for source in [
        "<div v-pre>{{ x }}<span @click='go'>{{ y }}</span></div>{{ z }}",
        "<div v-pre><br>{{ literal }}<i v-pre>{{ nested }}</i></div>",
        "<x v-pre>{{ text }}</x><p>{{ expr }}</p>",
    ] {
        assert_parser_parity(source, false);
    }
    assert_parser_parity("<!DOCTYPE html><main v-pre>{{ raw }}</main>", true);
}
