//! `css/*` rules that read the L1 style structure (#11550–#11553).
//!
//! A [`StyleRule`] checks the authored structure `vize_l1` reads for every
//! style language (CSS, SCSS, Sass, Less and Stylus): rules, at-rules and
//! declarations with file-absolute spans, and the comments beside them.
//! Nothing is preprocessed, so a finding points at the authored bytes of the
//! block in any language. Rules move here from the lightningcss lane
//! ([`super::CssRule`]) one at a time.

use cssparser::{Parser, ParserInput, Token};
use vize_l0::{Span, String};
use vize_l1::css::{StyleNode, StyleNodeKind, StyleSheet};

use super::{CssLintResult, CssRuleMeta};

/// Whether one authored CSS identifier decodes to `expected`.
///
/// The fast path covers ordinary names without constructing a parser. Escaped
/// names use cssparser so rules compare CSS values while retaining the
/// authored span for diagnostics.
pub(super) fn identifier_eq(text: &str, expected: &str) -> bool {
    if !text.as_bytes().contains(&b'\\') {
        return text.eq_ignore_ascii_case(expected);
    }
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    parser
        .expect_ident()
        .is_ok_and(|name| name.eq_ignore_ascii_case(expected))
        && parser.is_exhausted()
}

/// The decoded value of one authored CSS identifier.
pub(super) fn decoded_identifier(text: &str) -> Option<String> {
    if !text.as_bytes().contains(&b'\\') {
        return Some(String::from(text));
    }
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let name = parser.expect_ident_cloned().ok()?;
    parser.is_exhausted().then(|| String::from(name.as_ref()))
}

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

/// At-rules whose blocks hold style rules the rules check, as the lightningcss
/// lane descended (`@media`, `@supports`, `@layer`), plus the preprocessor
/// blocks that emit style rules where they are used (Sass spells `@mixin` and
/// `@include` as `=` and `+`).
const STYLE_SCOPES: [&str; 13] = [
    "media", "supports", "layer", "mixin", "include", "if", "else", "each", "for", "while",
    "at-root", "=", "+",
];

/// Whether `node` is in the scope the rule lane checks: under style rules and
/// the [`STYLE_SCOPES`] at-rules only, not in `@keyframes`, `@font-face`,
/// `@page`, `@container` or another at-rule the lightningcss lane skipped.
pub(super) fn in_style_scope(sheet: &StyleSheet<'_>, node: &StyleNode) -> bool {
    let nodes = sheet.nodes();
    let mut parent = node.parent();
    while let Some(index) = parent {
        let Some(ancestor) = nodes.get(index) else {
            return false;
        };
        if let StyleNodeKind::AtRule { name } = sheet.kind(ancestor) {
            let name = sheet.text(name);
            if !STYLE_SCOPES
                .iter()
                .any(|scope| name.eq_ignore_ascii_case(scope))
            {
                return false;
            }
        }
        parent = ancestor.parent();
    }
    true
}

/// Every rule's selector span, in any scope, in document order.
pub(super) fn all_rule_preludes<'s>(sheet: &'s StyleSheet<'_>) -> impl Iterator<Item = Span> + 's {
    sheet
        .nodes()
        .iter()
        .filter(|node| sheet.kind(node) == StyleNodeKind::Rule)
        .map(|node| node.prelude())
}

/// Every rule's selector span in the checked scope, in document order.
pub(super) fn rule_preludes<'s>(sheet: &'s StyleSheet<'_>) -> impl Iterator<Item = Span> + 's {
    sheet
        .nodes()
        .iter()
        .filter(|node| sheet.kind(node) == StyleNodeKind::Rule && in_style_scope(sheet, node))
        .map(|node| node.prelude())
}

/// The file-absolute span of each `#id` in the key (rightmost) compound of
/// each selector of a list, in the order lightningcss's selector iterator
/// visits them (right to left within the compound): the scope
/// `css/no-id-selectors` has always had. Arguments of functional
/// pseudo-classes (`:not(#a)`) and attribute values (`[href="#a"]`) are not
/// selectors of the rule, and interpolation (`#{…}`) is not an ID token.
/// Line comments are masked first, so a `//` comment cannot open a block.
pub(super) fn key_compound_ids(sheet: &StyleSheet<'_>, prelude: Span) -> Vec<Span> {
    let text = masked(sheet, prelude);
    let mut input = ParserInput::new(&text);
    let mut parser = Parser::new(&mut input);
    let mut ids = Vec::new();
    let mut compound: Vec<Span> = Vec::new();
    // A combinator was read: the next compound token starts a new compound.
    let mut combined = false;
    let flush = |compound: &mut Vec<Span>, ids: &mut Vec<Span>| {
        ids.extend(compound.drain(..).rev());
    };
    loop {
        let start = parser.position().byte_index();
        let Ok(token) = parser.next_including_whitespace_and_comments() else {
            break;
        };
        match token {
            Token::WhiteSpace(_) | Token::Delim('>' | '+' | '~') => combined = true,
            Token::Comment(_) => {}
            Token::Comma => {
                flush(&mut compound, &mut ids);
                combined = false;
            }
            token => {
                if combined {
                    compound.clear();
                    combined = false;
                }
                match token {
                    Token::IDHash(_) => {
                        let end = parser.position().byte_index();
                        compound.push(Span::new(
                            prelude.start + start as u32,
                            prelude.start + end as u32,
                        ));
                    }
                    Token::Function(_)
                    | Token::ParenthesisBlock
                    | Token::SquareBracketBlock
                    | Token::CurlyBracketBlock => skip_block(&mut parser),
                    _ => {}
                }
            }
        }
    }
    flush(&mut compound, &mut ids);
    ids
}

/// The text of `span` with the block's preprocessor comments (`// …`, and an
/// indented Sass `/*` without `*/`) replaced by spaces of the same byte
/// length, so token offsets stay authored offsets. A terminated `/* … */`
/// stays a comment token, which separates nothing: `#a/**/.b` is one
/// compound. Comments are in document order, so only those overlapping
/// `span` are visited.
pub(super) fn masked(sheet: &StyleSheet<'_>, span: Span) -> String {
    let mut text = String::from(sheet.text(span));
    let comments = sheet.comments();
    let first = comments.partition_point(|comment| comment.span().end <= span.start);
    let inside = comments
        .get(first..)
        .unwrap_or_default()
        .iter()
        .map(|comment| comment.span())
        .take_while(|comment| comment.start < span.end);
    for comment in inside {
        let body = sheet.text(comment);
        if body.starts_with("/*") && body.len() >= 4 && body.ends_with("*/") {
            continue;
        }
        let from = comment.start.max(span.start) - span.start;
        let to = comment.end.min(span.end) - span.start;
        if text.get(from as usize..to as usize).is_some() {
            let mut blank = String::default();
            for _ in from..to {
                blank.push(' ');
            }
            text.replace_range(from as usize..to as usize, &blank);
        }
    }
    text
}

/// The file-absolute span of each terminal `!important` flag in `value`: at
/// the top level of the value, followed only by other `!flag`s (Sass
/// `!default`). Strings, blocks and `/* … */` comments are CSS tokens; a flag
/// inside one of the block's comments (`//` in a preprocessor) is skipped.
pub(super) fn important_flags(sheet: &StyleSheet<'_>, value: Span) -> Vec<Span> {
    if !sheet.text(value).as_bytes().contains(&b'!') {
        return Vec::new();
    }
    let text = masked(sheet, value);
    let mut input = ParserInput::new(&text);
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
        flags.push(Span::new(
            value.start + start as u32,
            value.start + end as u32,
        ));
    }
    flags
}

/// After a function or bracketed block token, its contents are read with
/// the next token unless consumed now, which would record that token's
/// start inside the block. Call it only right after such a token.
pub(super) fn skip_block(parser: &mut Parser<'_, '_>) {
    let _: Result<(), cssparser::ParseError<'_, ()>> = parser.parse_nested_block(|nested| {
        while nested.next_including_whitespace_and_comments().is_ok() {}
        Ok(())
    });
}

#[cfg(test)]
mod tests;
