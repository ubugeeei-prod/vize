//! Sass (indented syntax) and Stylus, where a selector opens a block by
//! indenting the lines after it.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use vize_l0::String;

use crate::style::classes::collect_classes_from_selector_prelude;
use crate::style::structure::{Construct, Interpolation, Lexis, StyleSheet};
use crate::style::css::{AuthoredClasses, StyleNodeKind};
use crate::style::nesting::{self, Dialect, resolve};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Syntax {
    Sass,
    Stylus,
}

pub(super) const SASS: Lexis = Lexis {
    line_comments: true,
    interpolation: Interpolation::Hash,
    indented: true,
    indented_comments: true,
    colonless_declarations: false,
    construct: sass_construct,
};

/// Stylus reads braces when any line opens one, indentation otherwise.
pub(super) fn stylus(source: &str) -> Lexis {
    Lexis {
        line_comments: true,
        interpolation: Interpolation::Brace,
        indented: !is_braced_stylus(source),
        indented_comments: false,
        colonless_declarations: true,
        construct: stylus_construct,
    }
}

/// Stylus accepts CSS braces. Outside comments, strings and unquoted
/// `url(…)` payloads, a block brace ends a line (`.a {`) or, in a one-line
/// rule (`.a { color: red; }`), follows whitespace and its `}` encloses a `;`
/// or a `: ` (in a string too: `.a {content:"a;b"}`); an interpolation
/// (`.item-{i}`, `{prop}: 1`) does neither. The choice is per block, so a
/// block that mixes braced and indented rules is read one way only.
fn is_braced_stylus(source: &str) -> bool {
    let code = code_lines(source);
    // `code` keeps every byte offset of `source`, so the same range of a
    // line of each is the same authored bytes.
    source.lines().zip(code.lines()).any(|(authored, line)| {
        if line.trim_end().ends_with('{') {
            return true;
        }
        let bytes = line.as_bytes();
        bytes.iter().enumerate().any(|(at, &byte)| {
            byte == b'{'
                && at > 0
                && bytes.get(at - 1).is_some_and(u8::is_ascii_whitespace)
                && line
                    .get(at + 1..)
                    .and_then(|rest| rest.find('}'))
                    .and_then(|close| authored.get(at + 1..at + 1 + close))
                    .is_some_and(|inner| inner.contains(';') || inner.contains(": "))
        })
    })
}

/// `source` with comments, string contents and unquoted `url(…)` payloads
/// blanked byte for byte, line breaks kept, so every offset is the authored
/// one. A payload is opaque as the structure lexer reads it: `//`, braces
/// and `;` in it are data.
fn code_lines(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = bytes.to_vec();
    let mut blank = |from: usize, to: usize| {
        for byte in code.get_mut(from..to).into_iter().flatten() {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    };
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        index = match byte {
            // An escaped character is opaque, as the structure lexer reads
            // it: `.quoted\"name` opens no string.
            b'\\' => {
                let end = char_end(bytes, index + 1);
                blank(index + 1, end);
                end
            }
            b'"' | b'\'' => {
                let close = string_close(bytes, index);
                blank(index + 1, close);
                close + 1
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let end = find(bytes, index, b"\n").unwrap_or(bytes.len());
                blank(index, end);
                end
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let end = find(bytes, index + 2, b"*/").map_or(bytes.len(), |at| at + 2);
                blank(index, end);
                end
            }
            b'u' | b'U' => match url_payload(bytes, index) {
                Some((from, close)) => {
                    blank(from, close);
                    close + 1
                }
                None => index + 1,
            },
            _ => index + 1,
        };
    }
    // Blanks start and end at ASCII bytes, so whole characters stay whole.
    core::str::from_utf8(&code)
        .map(String::from)
        .unwrap_or_default()
}

/// The index of the quote closing the string opened at `open`, or the end.
fn string_close(bytes: &[u8], open: usize) -> usize {
    let quote = bytes.get(open).copied();
    let mut cursor = open + 1;
    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'\\' {
            cursor = char_end(bytes, cursor + 1);
            continue;
        }
        if Some(byte) == quote {
            return cursor;
        }
        cursor += 1;
    }
    bytes.len()
}

/// The payload start and the index of the `)` (or the end) of an unquoted
/// `url(` that starts at `index`.
fn url_payload(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let starts_name = index
        .checked_sub(1)
        .and_then(|previous| bytes.get(previous))
        .is_none_or(|&previous| {
            !(previous.is_ascii_alphanumeric()
                || matches!(previous, b'-' | b'_')
                || !previous.is_ascii())
        });
    let named = bytes
        .get(index..index + 4)
        .is_some_and(|name| name.eq_ignore_ascii_case(b"url("));
    if !starts_name || !named {
        return None;
    }
    let mut from = index + 4;
    while bytes.get(from).is_some_and(u8::is_ascii_whitespace) {
        from += 1;
    }
    if matches!(bytes.get(from), Some(b'\'' | b'"')) {
        return None;
    }
    let mut cursor = from;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b')' => return Some((from, cursor)),
            b'\\' => cursor = char_end(bytes, cursor + 1),
            _ => cursor += 1,
        }
    }
    Some((from, bytes.len()))
}

/// The end of the character starting at `index` (the end of `bytes` past it).
fn char_end(bytes: &[u8], index: usize) -> usize {
    if index >= bytes.len() {
        return bytes.len();
    }
    let mut end = index + 1;
    while bytes.get(end).is_some_and(|&byte| byte & 0xC0 == 0x80) {
        end += 1;
    }
    end
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// `=name` and `+name` spell `@mixin` and `@include`.
fn sass_construct(text: &str) -> Construct {
    let bytes = text.as_bytes();
    if matches!(bytes.first(), Some(b'=' | b'+'))
        && bytes
            .get(1)
            .is_some_and(|&byte| byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_'))
    {
        Construct::Sigil
    } else {
        Construct::Shared
    }
}

/// Control flow, block mixins (`+name()`), calls and function definitions
/// (`name(…)`) and assignments (`name = value`) are Stylus, not CSS.
fn stylus_construct(text: &str) -> Construct {
    let keyword = text
        .split(|c: char| c.is_ascii_whitespace() || c == '(')
        .next()
        .unwrap_or_default();
    let name_end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '$')))
        .unwrap_or(text.len());
    let rest = text.get(name_end..).unwrap_or_default().trim_start();
    let assigns = name_end > 0
        && (rest.starts_with('=') && !rest.starts_with("==")
            || ["?=", ":=", "+=", "-=", "*=", "/="]
                .iter()
                .any(|operator| rest.starts_with(operator)));
    if text.starts_with('+')
        || matches!(keyword, "if" | "unless" | "for" | "else" | "return")
        || starts_call(text)
        || assigns
    {
        Construct::Other
    } else {
        Construct::Shared
    }
}

/// Class names the rules declare. Stylus written with braces reads like SCSS,
/// but its mixin calls are not modeled there, so that result is incomplete.
pub(super) fn authored_classes(sheet: &StyleSheet<'_>, syntax: Syntax) -> Option<AuthoredClasses> {
    if syntax == Syntax::Stylus && is_braced_stylus(sheet.source()) {
        return nested::authored_classes(sheet, Dialect::Scss)
            .map(|found| AuthoredClasses::new(found.into_names(), false));
    }
    let mut classes = BTreeSet::new();
    let mut complete = true;
    // The resolved selectors of every node that opens a block; `None` is an
    // unknown context.
    let mut resolved: Vec<Option<Vec<String>>> = Vec::with_capacity(sheet.nodes().len());
    for node in sheet.nodes() {
        // A declaration adds no rules.
        if node.block().is_none() && matches!(sheet.kind(node), StyleNodeKind::Declaration { .. }) {
            resolved.push(None);
            continue;
        }
        let parent = match node.parent() {
            Some(parent) => resolved.get(parent)?.clone(),
            None => Some(Vec::new()),
        };
        let known = parent.is_some();
        let text = sheet.code(node.prelude());
        if node.block().is_none() {
            if known && statement_adds_rules(&text, syntax) {
                complete = false;
            }
            resolved.push(None);
            continue;
        }
        let (selectors, hides) = match line(&text, syntax) {
            Line::Selector(selector) => {
                let selectors = resolve(
                    selector.as_bytes(),
                    parent,
                    node.is_interpolated(),
                    Dialect::Scss,
                );
                let hides = nested::block_hides_rules(
                    selector.as_bytes(),
                    selectors.is_some(),
                    Dialect::Scss,
                );
                (selectors, hides)
            }
            Line::Definition => (None, false),
            Line::Unknown => (None, true),
        };
        if known && hides {
            complete = false;
        }
        for selector in selectors.iter().flatten() {
            let mut rule = BTreeSet::new();
            if collect_classes_from_selector_prelude(selector.as_bytes(), &mut rule).is_some() {
                classes.extend(rule);
            } else {
                complete = false;
            }
        }
        resolved.push(selectors);
    }
    Some(AuthoredClasses::new(classes, complete))
}

/// What a block-opening line spells.
enum Line<'t> {
    /// A selector or an at-rule.
    Selector(&'t str),
    /// A mixin or function definition: its body emits nothing here.
    Definition,
    /// A block whose rules cannot be named: control flow, a Stylus block
    /// mixin call or partial parent reference.
    Unknown,
}

/// Whether a statement line can add rules: an include, an import or, in
/// Stylus, a mixin call.
fn statement_adds_rules(text: &str, syntax: Syntax) -> bool {
    text.starts_with('+')
        || syntax == Syntax::Stylus && starts_call(text)
        || nested::statement_adds_rules(text.as_bytes(), Dialect::Scss)
}

/// `name(`: a Stylus mixin or function, defined when it opens a block.
fn starts_call(text: &str) -> bool {
    text.find('(')
        .and_then(|open| text.get(..open))
        .is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
}

fn line(text: &str, syntax: Syntax) -> Line<'_> {
    match syntax {
        Syntax::Sass => {
            if text.starts_with('=') {
                return Line::Definition;
            }
            // `+mixin` content blocks keep the enclosing selector.
            if text.starts_with('+') {
                return Line::Selector("@include");
            }
            Line::Selector(text)
        }
        Syntax::Stylus => {
            if starts_call(text) {
                return Line::Definition;
            }
            let keyword = text.split_ascii_whitespace().next().unwrap_or_default();
            if text.starts_with('+')
                || matches!(keyword, "if" | "unless" | "for" | "else")
                || text.contains(['^', '~'])
                || text.contains("../")
            {
                return Line::Unknown;
            }
            Line::Selector(text)
        }
    }
}
