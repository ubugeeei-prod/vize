//! Test-only original first-scalar Armature consumer on the public compatibility sink.

use super::attribute_entity_sink::OriginalFirstScalarSink;
use super::{Parser, callbacks::ParserCallbacks, current_whitespace_strategy};
use vize_l0::{Allocator, Vec};
use vize_l1::markup::{Component, Delimiters, Document, LexOptions, Lexer};
use vize_relief::{
    RootNode,
    errors::CompilerError,
    options::{ParserOptions, WhitespaceStrategy},
};

pub(super) fn parse_with_compat<'a>(
    mut parser: Parser<'a>,
) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
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
            OriginalFirstScalarSink::new(ParserCallbacks {
                parser: &mut parser,
            }),
            options,
        )
        .run();
    } else {
        Lexer::<Component, _>::new(
            source,
            OriginalFirstScalarSink::new(ParserCallbacks {
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
