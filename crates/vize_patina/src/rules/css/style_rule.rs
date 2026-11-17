//! `css/*` rules that read the L1 style structure (#11550–#11553).
//!
//! A [`StyleRule`] checks the authored structure `vize_l1` reads for every
//! style language (CSS, SCSS, Sass, Less and Stylus): rules, at-rules and
//! declarations with file-absolute spans, and the comments beside them.
//! Nothing is preprocessed, so a finding points at the authored bytes of the
//! block in any language. Rules move here from the lightningcss lane
//! ([`super::CssRule`]) one at a time.

use cssparser::{Parser, ParserInput, Token};
use vize_l0::Span;
use vize_l1::css::{StyleNodeKind, StyleSheet};

use super::{CssLintResult, CssRuleMeta};

/// A `css/*` rule over the L1 structure of one style block.
pub trait StyleRule: Send + Sync {
    /// Get rule metadata.
    fn meta(&self) -> &'static CssRuleMeta;

    /// Check one block. Spans of `sheet` are file-absolute.
    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult);
}

/// Every declaration's value span, in document order. Declarations inside a
/// custom property's value (`--raw: { a: b }`) are its opaque text.
pub(super) fn declaration_values<'s>(sheet: &'s StyleSheet<'_>) -> impl Iterator<Item = Span> + 's {
    let nodes = sheet.nodes();
    let custom = |index: usize| {
        nodes.get(index).is_some_and(|node| {
            matches!(sheet.kind(node), StyleNodeKind::Declaration { name, .. }
                if sheet.text(name).starts_with("--"))
        })
    };
    nodes.iter().filter_map(move |node| {
        let mut parent = node.parent();
        while let Some(index) = parent {
            if custom(index) {
                return None;
            }
            parent = nodes.get(index).and_then(|node| node.parent());
        }
        match sheet.kind(node) {
            StyleNodeKind::Declaration { value, .. } => Some(value),
            _ => None,
        }
    })
}

/// The file-absolute span of each terminal `!important` flag in `value`: at
/// the top level of the value, followed only by other `!flag`s (Sass
/// `!default`). Strings, blocks and `/* … */` comments are CSS tokens; a flag
/// inside one of the block's comments (`//` in a preprocessor) is skipped.
pub(super) fn important_flags(sheet: &StyleSheet<'_>, value: Span) -> Vec<Span> {
    let text = sheet.text(value);
    if !text.as_bytes().contains(&b'!') {
        return Vec::new();
    }
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let mut flags = Vec::new();
    loop {
        let start = parser.position().byte_index();
        let Ok(token) = parser.next_including_whitespace_and_comments() else {
            break;
        };
        match token {
            Token::Delim('!') => {}
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                skip_block(&mut parser);
                continue;
            }
            _ => continue,
        }
        let important = parser.try_parse(|parser| {
            parser.expect_ident_matching("important")?;
            let end = parser.position().byte_index();
            // Only further flags may follow.
            while !parser.is_exhausted() {
                parser.expect_delim('!')?;
                parser.expect_ident()?;
            }
            Ok::<_, cssparser::BasicParseError<'_>>(end)
        });
        let Ok(end) = important else {
            continue;
        };
        let span = Span::new(value.start + start as u32, value.start + end as u32);
        let commented = sheet
            .comments()
            .iter()
            .any(|comment| comment.span().start <= span.start && span.end <= comment.span().end);
        if !commented {
            flags.push(span);
        }
    }
    flags
}

/// After a function or bracketed block token, its contents are read with
/// the next token unless consumed now, which would record that token's
/// start inside the block. Call it only right after such a token.
fn skip_block(parser: &mut Parser<'_, '_>) {
    let _: Result<(), cssparser::ParseError<'_, ()>> = parser.parse_nested_block(|nested| {
        while nested.next_including_whitespace_and_comments().is_ok() {}
        Ok(())
    });
}

#[cfg(test)]
mod tests;
