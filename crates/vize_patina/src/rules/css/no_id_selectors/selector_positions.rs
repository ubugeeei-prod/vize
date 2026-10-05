//! Authored ID token spans within an already parsed style rule's prelude.

use cssparser::{CowRcStr, ParseError, Parser, ParserInput, Token};
use lightningcss::rules::Location;

struct IdToken<'i> {
    name: CowRcStr<'i>,
    start: usize,
    end: usize,
    used: bool,
}

pub(super) struct SelectorPositions<'i> {
    selectors: Vec<Vec<IdToken<'i>>>,
}

impl<'i> SelectorPositions<'i> {
    pub(super) fn new(source: &'i str, location: Location) -> Self {
        let mut selectors = vec![Vec::new()];
        let Some(base) = source_position(source, location) else {
            return Self { selectors };
        };
        let Some(prelude) = source.get(base..) else {
            return Self { selectors };
        };
        let mut input = ParserInput::new(prelude);
        let mut parser = Parser::new(&mut input);
        loop {
            let start = base + parser.position().byte_index();
            let Ok(token) = parser.next_including_whitespace_and_comments().cloned() else {
                break;
            };
            match token {
                Token::CurlyBracketBlock => break,
                Token::Comma => selectors.push(Vec::new()),
                Token::IDHash(name) => {
                    if let Some(selector) = selectors.last_mut() {
                        selector.push(IdToken {
                            name,
                            start,
                            end: base + parser.position().byte_index(),
                            used: false,
                        });
                    }
                }
                Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock => {
                    // Consume the whole block now: otherwise next() skips it
                    // after the next token's start position has been recorded.
                    let _: Result<(), ParseError<'_, ()>> = parser.parse_nested_block(|nested| {
                        while nested.next().is_ok() {}
                        Ok(())
                    });
                }
                _ => {}
            }
        }
        Self { selectors }
    }

    pub(super) fn take(&mut self, index: usize, name: &str, offset: usize) -> Option<(u32, u32)> {
        // The accepted selector iterator walks matching order, from the right.
        // Keep comma entries separate, and consume repeated IDs only once.
        let token = self
            .selectors
            .get_mut(index)?
            .iter_mut()
            .rev()
            .find(|token| !token.used && token.name.as_ref() == name)?;
        token.used = true;
        Some((
            u32::try_from(offset.checked_add(token.start)?).ok()?,
            u32::try_from(offset.checked_add(token.end)?).ok()?,
        ))
    }
}

fn source_position(source: &str, location: Location) -> Option<usize> {
    let (mut line, mut column) = (0, 1);
    let mut chars = source.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if line == location.line && column == location.column {
            return Some(index);
        }
        match ch {
            '\r' | '\n' | '\u{c}' => {
                if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                    chars.next();
                }
                line += 1;
                column = 1;
            }
            _ => column += ch.len_utf16() as u32,
        }
    }
    (line == location.line && column == location.column).then_some(source.len())
}
