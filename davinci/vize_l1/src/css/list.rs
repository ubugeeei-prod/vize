//! Class lists from the same original rule tokens, without another parse.

use super::{StyleIssue, StyleIssueCode, StyleSyntax};
use cssparser::Token;

/// Unforgeable receipt for a complete comma-separated simple-class selector.
/// Every insertion is the end of an actual original classname token.
///
/// ```compile_fail
/// use vize_l1::css::SimpleClassListStyle;
/// let _ = SimpleClassListStyle { count: 2, insertions: [1, 2] };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SimpleClassListStyle<'o, 'a> {
    syntax: &'o StyleSyntax<'a>,
    count: usize,
}

impl<'o, 'a> SimpleClassListStyle<'o, 'a> {
    #[must_use]
    pub const fn syntax(self) -> &'o StyleSyntax<'a> {
        self.syntax
    }
    #[must_use]
    pub const fn class_count(self) -> usize {
        self.count
    }
    /// These offsets borrow the validated actual token sequence; no caller
    /// list, reparsed selector or normalized spelling supplies authority.
    pub fn insertions(self) -> impl Iterator<Item = u32> + 'o {
        self.syntax
            .rule()
            .into_iter()
            .flat_map(|rule| rule.prelude().iter())
            .filter_map(|token| {
                matches!(token.token(), Token::Ident(_)).then_some(token.span().end)
            })
    }
}

pub(super) fn observe<'o, 'a>(
    syntax: &'o StyleSyntax<'a>,
) -> Result<SimpleClassListStyle<'o, 'a>, StyleIssue> {
    if let Some(issue) = syntax.issue() {
        return Err(issue);
    }
    let unsupported = || StyleIssue {
        code: StyleIssueCode::UnsupportedSelector,
        span: syntax.source().span(),
    };
    let rule = syntax.rule().ok_or_else(unsupported)?;
    let mut tokens = rule.prelude().iter().peekable();
    while tokens
        .peek()
        .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_) | Token::Comment(_)))
    {
        tokens.next();
    }
    let mut count = 0;
    loop {
        let dot = tokens.next().ok_or_else(unsupported)?;
        let name = tokens.next().ok_or_else(unsupported)?;
        let Token::Ident(decoded) = name.token() else {
            return Err(unsupported());
        };
        if !matches!(dot.token(), Token::Delim('.'))
            || syntax
                .source()
                .root_source()
                .get(name.span().start as usize..name.span().end as usize)
                != Some(decoded.as_ref())
        {
            return Err(unsupported());
        }
        count += 1;
        let mut whitespace = false;
        while tokens
            .peek()
            .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_)))
        {
            whitespace = true;
            tokens.next();
        }
        match tokens.next() {
            None if count >= 2 => return Ok(SimpleClassListStyle { syntax, count }),
            Some(comma) if !whitespace && matches!(comma.token(), Token::Comma) => {
                while tokens
                    .peek()
                    .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_)))
                {
                    tokens.next();
                }
            }
            _ => return Err(unsupported()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SurfaceParseOptions,
        container::{Vue, vue::DescriptorOptions},
        css::{StyleIssueCode, StyleSyntax},
    };
    use cssparser::Token;
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
    };

    fn options() -> DescriptorOptions {
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        }
    }

    #[test]
    fn class_list_receipts_borrow_every_actual_original_token_end_once() {
        let source = "<!-- 雪🌸 -->\r\n<style scoped>/* original */ .雪,\r\n .b,.c { content:'雪'; }</style><template><p/></template>";
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(&arena, source, options());
        let original = descriptor.admitted().unwrap().styles().next().unwrap();
        let syntax = StyleSyntax::observe(original);
        assert!(syntax.simple_class().is_err());
        let receipt = syntax.simple_class_list().unwrap();
        assert_eq!(receipt.class_count(), 3);
        assert!(core::ptr::eq(receipt.syntax(), &syntax));
        assert!(core::ptr::eq(syntax.source().root_source(), source));
        let expected = syntax.rule().unwrap().prelude().iter().filter_map(|token| {
            matches!(token.token(), Token::Ident(_)).then_some(token.span().end)
        });
        assert!(receipt.insertions().eq(expected));
        for (end, name) in receipt.insertions().zip(["雪", "b", "c"]) {
            assert_eq!(&source[end as usize - name.len()..end as usize], name);
            assert!(
                syntax
                    .source()
                    .contains_block_span(vize_l0::Span::new(end, end))
            );
        }
        let moved = syntax;
        assert_eq!(moved.simple_class_list().unwrap().class_count(), 3);
        assert!(core::ptr::eq(moved.source().root_source(), source));
    }

    #[test]
    fn every_unproven_list_shape_retains_original_syntax_and_refuses_whole_receipt() {
        for css in [
            ".a{}",
            ".a,.b:hover{}",
            ".a,p{}",
            ".a,,.b{}",
            ".a,.b,{}",
            ".a,. b{}",
            ".a , .b{}",
            ".a/* comment */,.b{}",
            ".a,/* comment */.b{}",
            ".a,.b/* comment */{}",
            ".\\61,.b{}",
            ".a,.b{color:v-bind(x)}",
            ".a,.b{content:'v-bind(x)'}",
            ".a,.b{content:'v/**/-bind(x)'}",
            ".a,.b{color:red",
            "@charset 'utf-8';.a,.b{}",
            ".a,.b{} .c{}",
        ] {
            let source = alloc::format!("<style scoped>{css}</style><template><p/></template>");
            let arena = Allocator::default();
            let descriptor = Vue.observe_descriptor(&arena, &source, options());
            let syntax =
                StyleSyntax::observe(descriptor.admitted().unwrap().styles().next().unwrap());
            assert!(syntax.simple_class_list().is_err(), "{css}");
            assert!(core::ptr::eq(
                syntax.source().root_source(),
                source.as_str()
            ));
            assert_eq!(syntax.source().source(), css);
            if let Some(error) = syntax.parser_error() {
                assert_eq!(
                    syntax.simple_class_list().unwrap_err(),
                    syntax.issue().unwrap()
                );
                assert!(matches!(
                    error.kind,
                    cssparser::ParseErrorKind::Basic(_) | cssparser::ParseErrorKind::Custom(_)
                ));
            } else {
                assert!(syntax.rule().is_some());
                assert_eq!(
                    syntax.simple_class_list().unwrap_err().code,
                    StyleIssueCode::UnsupportedSelector
                );
            }
        }
    }
}
