//! Sass (indented syntax) and Stylus, where a selector opens a block by
//! indenting the lines after it.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use vize_l0::String;

use super::super::classes::collect_classes_from_selector_prelude;
use super::super::structure::{Construct, Interpolation, Lexis, StyleSheet};
use super::super::{AuthoredClasses, StyleNodeKind};
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
