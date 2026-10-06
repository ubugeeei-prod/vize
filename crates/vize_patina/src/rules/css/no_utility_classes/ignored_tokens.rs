//! CSS-token-owned comments and literal values excluded from raw class matches.

use cssparser::{ParseError, Parser, ParserInput, Token};

pub(super) struct IgnoredTokens {
    ranges: Vec<(usize, usize)>,
}

impl IgnoredTokens {
    pub(super) fn new(source: &str) -> Self {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        let mut ranges = Vec::new();
        collect(&mut parser, &mut ranges);
        Self { ranges }
    }

    pub(super) fn contains(&self, position: usize) -> bool {
        let index = self.ranges.partition_point(|(start, _)| *start <= position);
        index
            .checked_sub(1)
            .and_then(|previous| self.ranges.get(previous))
            .is_some_and(|(_, end)| position < *end)
    }
}

fn collect<'i>(parser: &mut Parser<'i, '_>, ranges: &mut Vec<(usize, usize)>) {
    loop {
        let start = parser.position().byte_index();
        let Ok(token) = parser.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        match token {
            Token::Comment(_)
            | Token::QuotedString(_)
            | Token::UnquotedUrl(_)
            | Token::BadString(_)
            | Token::BadUrl(_) => ranges.push((start, parser.position().byte_index())),
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                // Explicitly enter blocks: the next outer token otherwise skips
                // their contents, including strings/comments in declarations.
                let _: Result<(), ParseError<'i, ()>> = parser.parse_nested_block(|nested| {
                    collect(nested, ranges);
                    Ok(())
                });
            }
            _ => {}
        }
    }
}
