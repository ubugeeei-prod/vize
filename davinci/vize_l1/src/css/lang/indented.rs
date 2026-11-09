//! Sass (indented syntax) and Stylus, where a selector opens a block by
//! indenting the lines after it.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use vize_l0::String;

use super::super::AuthoredClasses;
use super::super::classes::collect_classes_from_selector_prelude;
use super::super::structure::{Construct, Interpolation, Lexis};
use super::nested::{self, Dialect, resolve};

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

fn is_braced_stylus(source: &str) -> bool {
    source.lines().any(|line| line.trim_end().ends_with('{'))
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
pub(super) fn authored_classes(source: &str, syntax: Syntax) -> Option<AuthoredClasses> {
    if syntax == Syntax::Stylus && is_braced_stylus(source) {
        return nested::authored_classes(source, Dialect::Scss)
            .map(|found| AuthoredClasses::new(found.into_names(), false));
    }
    let lines = code_lines(source)?;
    let mut classes = BTreeSet::new();
    let mut complete = true;
    // Open blocks: their indentation and resolved selectors (`None` is unknown).
    let mut blocks: Vec<(usize, Option<Vec<String>>)> = Vec::new();
    let mut index = 0usize;
    while let Some(&(indent, ref first)) = lines.get(index) {
        // A selector list may continue over lines that end with `,`.
        let mut text = String::from(first.as_str());
        while text.ends_with(',')
            && let Some((_, next)) = lines.get(index + 1)
        {
            text.push(' ');
            text.push_str(next);
            index += 1;
        }
        index += 1;
        let opens_block = lines.get(index).is_some_and(|(next, _)| *next > indent);
        while blocks.last().is_some_and(|(open, _)| *open >= indent) {
            blocks.pop();
        }
        let known = blocks.last().is_none_or(|(_, parent)| parent.is_some());
        if !opens_block {
            if known && statement_adds_rules(&text, syntax) {
                complete = false;
            }
            continue;
        }
        let parent = match blocks.last() {
            Some((_, parent)) => parent.clone(),
            None => Some(Vec::new()),
        };
        let (resolved, hides) = match line(&text, syntax) {
            Line::Selector(selector, interpolated) => {
                let resolved = resolve(selector.as_bytes(), parent, interpolated, Dialect::Scss);
                let hides = nested::block_hides_rules(
                    selector.as_bytes(),
                    resolved.is_some(),
                    Dialect::Scss,
                );
                (resolved, hides)
            }
            Line::Definition => (None, false),
            Line::Unknown => (None, true),
        };
        if known && hides {
            complete = false;
        }
        for selector in resolved.iter().flatten() {
            let mut rule = BTreeSet::new();
            if collect_classes_from_selector_prelude(selector.as_bytes(), &mut rule).is_some() {
                classes.extend(rule);
            } else {
                complete = false;
            }
        }
        blocks.push((indent, resolved));
    }
    Some(AuthoredClasses::new(classes, complete))
}

/// What a block-opening line spells.
enum Line<'t> {
    /// A selector or an at-rule, with whether it interpolates.
    Selector(&'t str, bool),
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
                return Line::Selector("@include", false);
            }
            Line::Selector(text, text.contains("#{"))
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
            Line::Selector(text, text.contains('{'))
        }
    }
}

/// Non-empty lines with their indentation, comments removed.
fn code_lines(source: &str) -> Option<Vec<(usize, String)>> {
    let mut lines = Vec::new();
    let mut in_block_comment = false;
    for line in source.lines() {
        let mut code = String::default();
        let mut rest = line;
        loop {
            if in_block_comment {
                let Some(end) = rest.find("*/") else {
                    break;
                };
                rest = rest.get(end + 2..)?;
                in_block_comment = false;
            }
            let Some(start) = comment_start(rest) else {
                code.push_str(rest);
                break;
            };
            code.push_str(rest.get(..start)?);
            if rest.get(start..)?.starts_with("//") {
                break;
            }
            rest = rest.get(start + 2..)?;
            in_block_comment = true;
        }
        let trimmed = code.trim_end();
        if trimmed.trim_start().is_empty() {
            continue;
        }
        let indent = trimmed.len() - trimmed.trim_start().len();
        lines.push((indent, String::from(trimmed.trim_start())));
    }
    Some(lines)
}

/// The first `//` or `/*` outside strings and parentheses (`url(http://…)`).
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut quote = None;
    let mut parens = 0usize;
    for (index, &byte) in bytes.iter().enumerate() {
        if let Some(open) = quote {
            if byte == open {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'/' if parens == 0 && matches!(bytes.get(index + 1), Some(b'/' | b'*')) => {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}
