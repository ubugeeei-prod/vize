//! Interpolation delimiter validation and tokenizer setup.

use super::{Parser, callbacks::ParserCallbacks};
use vize_l0::{ErrorCode, Vec};
use vize_l1::markup::lex::compat::adapter::CompatSink;
use vize_l1::markup::{Component, Delimiters, Document, LexOptions, Lexer};
use vize_relief::{RootNode, errors::CompilerError};

impl<'a> Parser<'a> {
    pub(super) fn tokenize_template(&mut self) -> bool {
        if self.options.delimiters.0.is_empty() || self.options.delimiters.1.is_empty() {
            let loc = self.create_loc(0, 0);
            self.errors.push(CompilerError::with_message(
                ErrorCode::ExtendPoint,
                "Interpolation delimiters must both be non-empty.",
                Some(loc),
            ));
            return false;
        }

        let delimiter_open: Vec<'a, u8> =
            Vec::from_iter_in(self.options.delimiters.0.bytes(), &self.allocator);
        let delimiter_close: Vec<'a, u8> =
            Vec::from_iter_in(self.options.delimiters.1.bytes(), &self.allocator);
        let document = self.document;
        let source = self.source;
        let options = LexOptions {
            delimiters: Delimiters {
                open: &delimiter_open,
                close: &delimiter_close,
            },
            in_tag_comments: self.options.experimental_in_tag_comments,
            raw_interpolation: {
                #[cfg(feature = "legacy")]
                {
                    self.raw_html_interpolation_enabled()
                }
                #[cfg(not(feature = "legacy"))]
                {
                    false
                }
            },
        };
        // Keep the original compatibility sink and lexer instantiation. The
        // private callback alone opts into complete borrowed attribute values.
        let sink = CompatSink::new(ParserCallbacks { parser: self });
        if document {
            Lexer::<Document, _>::new(source, sink, options).run();
        } else {
            Lexer::<Component, _>::new(source, sink, options).run();
        }
        true
    }

    pub(super) fn into_result(mut self) -> (RootNode<'a>, std::vec::Vec<CompilerError>) {
        let root = self
            .root
            .take()
            .unwrap_or_else(|| RootNode::new(self.allocator, self.source));
        (root, self.errors)
    }

    #[cfg(feature = "legacy")]
    fn raw_html_interpolation_enabled(&self) -> bool {
        crate::legacy::LegacyDialectCapabilities::for_dialect(self.options.dialect)
            .raw_html_interpolation
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_with_options;
    use vize_l0::{Allocator, ErrorCode};
    use vize_relief::options::ParserOptions;

    #[test]
    fn empty_interpolation_delimiters_are_rejected_without_partial_output() {
        for (open, close) in [("", "}}"), ("{{", "")] {
            let allocator = Allocator::new();
            let options = ParserOptions {
                delimiters: (open.into(), close.into()),
                ..ParserOptions::default()
            };

            let (root, errors) = parse_with_options(&allocator, "<p>{{ value }}</p>", options);

            assert!(root.children.is_empty(), "{open:?}, {close:?}");
            assert_eq!(errors.len(), 1, "{open:?}, {close:?}: {errors:?}");
            assert_eq!(errors[0].code, ErrorCode::ExtendPoint);
            assert_eq!(
                errors[0].message.as_str(),
                "Interpolation delimiters must both be non-empty."
            );
        }
    }
}
