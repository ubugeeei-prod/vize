//! The TSRX markup grammar: elements, fragments, attributes, text, comments
//! and expression children. Markup bytes are scanned here; every embedded
//! expression is parsed by the host parser through the cursor.

use core::cell::RefCell;

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{Kind, MarkupCursor, MarkupGrammar as HostGrammar, MarkupSite};
use oxc_span::GetSpan;
use vize_l0::{Span, Vec};

use super::diagnostic as error;
use super::tree::Node;
use super::{Region, RegionSite};

/// The registered host grammar. Regions are recorded in completion order;
/// nested regions (markup inside an embedded expression) complete first.
pub(super) struct Grammar<'a> {
    pub(super) regions: RefCell<Vec<'a, Region<'a>>>,
}

impl<'a> HostGrammar<'a> for Grammar<'a> {
    fn opens(&self, source: &'a str, offset: u32, site: MarkupSite) -> bool {
        opens(source.as_bytes(), offset, site)
    }

    fn region(&self, cursor: &mut MarkupCursor<'_, 'a>, site: MarkupSite) -> u32 {
        let start = cursor.token().start();
        let mut scan = Scan {
            source: cursor.source(),
            alloc: cursor.allocator(),
            cursor,
        };
        let node = match site {
            MarkupSite::FunctionBody => scan.code_block(start, None),
            MarkupSite::Expression => scan.node(start),
        };
        let Some(node) = node else { return start };
        let span = node.span();
        let site = match site {
            MarkupSite::FunctionBody => RegionSite::FunctionBody,
            MarkupSite::Expression => RegionSite::Expression,
        };
        self.regions.borrow_mut().push(Region { span, site, node });
        span.end
    }
}

/// Whether an `@` at `offset` opens a statement container or a directive.
pub(super) fn opens(bytes: &[u8], offset: u32, site: MarkupSite) -> bool {
    let at = offset as usize;
    if bytes.get(at) != Some(&b'@') {
        return false;
    }
    if bytes.get(at + 1) == Some(&b'{') {
        return true;
    }
    if site == MarkupSite::FunctionBody {
        return false;
    }
    let Some(word) = word(bytes, offset + 1) else {
        return false;
    };
    let next = skip_trivia(bytes, offset + 1 + word.len() as u32) as usize;
    match word {
        b"if" | b"switch" => bytes.get(next) == Some(&b'('),
        b"try" => bytes.get(next) == Some(&b'{'),
        b"for" => {
            bytes.get(next) == Some(&b'(')
                || (word_is(bytes, next as u32, b"await")
                    && bytes.get(skip_trivia(bytes, next as u32 + 5) as usize) == Some(&b'('))
        }
        _ => false,
    }
}

pub(super) fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') || byte >= 0x80
}

pub(super) fn is_identifier_part(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

/// The ASCII identifier word at `offset`, if any.
pub(super) fn word(bytes: &[u8], offset: u32) -> Option<&[u8]> {
    let start = offset as usize;
    let rest = bytes.get(start..)?;
    let len = rest
        .iter()
        .take_while(|byte| is_identifier_part(**byte))
        .count();
    (len > 0 && rest.first().is_some_and(|byte| is_identifier_start(*byte)))
        .then(|| rest.get(..len))
        .flatten()
}

/// Whether `expected` is written at `offset` and is not followed by an identifier part.
pub(super) fn word_is(bytes: &[u8], offset: u32, expected: &[u8]) -> bool {
    let start = offset as usize;
    bytes.get(start..start + expected.len()) == Some(expected)
        && !bytes
            .get(start + expected.len())
            .is_some_and(|byte| is_identifier_part(*byte))
}

/// Skip whitespace and comments.
pub(super) fn skip_trivia(bytes: &[u8], mut offset: u32) -> u32 {
    loop {
        let at = offset as usize;
        match (bytes.get(at), bytes.get(at + 1)) {
            (Some(byte), _) if byte.is_ascii_whitespace() => offset += 1,
            (Some(b'/'), Some(b'*')) => match find(bytes, at + 2, b"*/") {
                Some(end) => offset = end as u32 + 2,
                None => return bytes.len() as u32,
            },
            (Some(b'/'), Some(b'/')) => {
                offset = line_end(bytes, at) as u32;
            }
            _ => return offset,
        }
    }
}

pub(super) fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    let rest = bytes.get(from..)?;
    rest.windows(needle.len())
        .position(|window| window == needle)
        .map(|index| from + index)
}

pub(super) fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes
        .get(from..)
        .and_then(|rest| rest.iter().position(|byte| matches!(byte, b'\n' | b'\r')))
        .map_or(bytes.len(), |index| from + index)
}

/// One region's scanning state over the host cursor.
pub(super) struct Scan<'c, 'p, 'a> {
    pub(super) cursor: &'c mut MarkupCursor<'p, 'a>,
    pub(super) source: &'a str,
    pub(super) alloc: &'a Allocator,
}

pub(super) fn span_of(node: &impl GetSpan) -> Span {
    let span = node.span();
    Span::new(span.start, span.end)
}

impl<'a> Scan<'_, '_, 'a> {
    pub(super) fn bytes(&self) -> &'a [u8] {
        self.source.as_bytes()
    }

    pub(super) fn byte(&self, offset: u32) -> Option<u8> {
        self.bytes().get(offset as usize).copied()
    }

    pub(super) fn text(&self, span: Span) -> &'a str {
        self.source
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default()
    }

    pub(super) fn fail<T>(&mut self, diagnostic: OxcDiagnostic) -> Option<T> {
        self.cursor.fatal(diagnostic);
        None
    }

    pub(super) fn seat(&mut self, offset: u32) -> Option<()> {
        self.cursor.seat(offset).then_some(())
    }

    pub(super) fn token(&self) -> (Kind, Span) {
        let token = self.cursor.token();
        (token.kind(), Span::new(token.start(), token.end()))
    }

    /// Require the current host token to be `kind`; return its span.
    pub(super) fn expect(&mut self, kind: Kind) -> Option<Span> {
        let (found, span) = self.token();
        if found == kind {
            Some(span)
        } else {
            self.fail(error::unexpected(span))
        }
    }

    /// Parse `{ expression }` whose `{` is at `open`; return the expression and the
    /// offset after `}`.
    pub(super) fn braced_expression(&mut self, open: u32) -> Option<(Expression<'a>, u32)> {
        self.seat(open + 1)?;
        let expression = self.cursor.expression();
        if self.cursor.failed() {
            return None;
        }
        let close = self.expect(Kind::RCurly)?;
        Some((expression, close.end))
    }

    /// A markup node at `start`: `<…` or an `@` construct.
    pub(super) fn node(&mut self, start: u32) -> Option<Node<'a>> {
        match self.byte(start) {
            Some(b'<') => self.markup(start),
            Some(b'@') => self.at_node(start),
            _ => self.fail(error::unexpected(Span::new(start, start + 1))),
        }
    }
}
