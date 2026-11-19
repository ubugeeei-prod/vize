//! SCSS and Less: braced rules that nest, with selectors resolved against
//! their parents (`.card { &__title {} }` declares `card__title`).
//!
//! Rules whose selector cannot be known statically (interpolation, mixin and
//! control-flow bodies) contribute nothing; their siblings still do.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use vize_l0::String;

use super::super::classes::collect_classes_from_selector_prelude;
use super::super::structure::{Interpolation, Lexis, StyleSheet, shared};
use super::super::{AuthoredClasses, StyleNodeKind};

/// A resolved selector list grows multiplicatively with nested comma lists.
const MAX_SELECTORS: usize = 256;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Dialect {
    Scss,
    Less,
}

pub(super) const SCSS: Lexis = Lexis {
    line_comments: true,
    interpolation: Interpolation::Hash,
    indented: false,
    indented_comments: false,
    colonless_declarations: false,
    construct: shared,
};

pub(super) const LESS: Lexis = Lexis {
    interpolation: Interpolation::At,
    ..SCSS
};

/// Class names the rules declare, or `None` when the source is unbalanced.
/// Less mixin definitions (`.m() {}`) emit no rules and add no names.
pub(super) fn authored_classes(
    sheet: &StyleSheet<'_>,
    dialect: Dialect,
) -> Option<AuthoredClasses> {
    if !sheet.is_balanced() || !sheet.is_terminated() {
        return None;
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
        let prelude = sheet.code(node.prelude());
        let prelude = prelude.as_bytes();
        if node.block().is_none() {
            if known && statement_adds_rules(prelude, dialect) {
                complete = false;
            }
            resolved.push(None);
            continue;
        }
        let selectors = resolve(prelude, parent, node.is_interpolated(), dialect);
        if known && block_hides_rules(prelude, selectors.is_some(), dialect) {
            complete = false;
        }
        for selector in selectors.iter().flatten() {
            // A rule outside the supported selector grammar adds no names.
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

/// Whether a block in a known context can hold rules this scan cannot name:
/// includes and control flow, interpolated or unresolvable selectors. Mixin
/// and function definitions, nested properties, keyframes and font faces
/// emit no rules of their own.
pub(super) fn block_hides_rules(prelude: &[u8], resolved: bool, dialect: Dialect) -> bool {
    let Ok(text) = core::str::from_utf8(prelude) else {
        return true;
    };
    let text = text.trim();
    if let Some((name, _)) = at_rule(text) {
        return name == "include" || !resolved && !emits_no_rules(name);
    }
    !resolved
        && !(dialect == Dialect::Scss && is_nested_property(text))
        && !(dialect == Dialect::Less
            && split_top_level_commas(text)
                .iter()
                .any(|part| is_less_mixin_definition(part)))
}

/// At-rules whose block is a definition or holds no selectors.
fn emits_no_rules(name: &str) -> bool {
    matches!(
        name,
        "mixin" | "function" | "font-face" | "page" | "property" | "counter-style"
    ) || name.ends_with("keyframes")
        // A Less detached ruleset (`@detached: { ... }`) runs where it is called.
        || name.ends_with(':')
}

/// Whether a statement in a known context can add rules: an include, an
/// import of another stylesheet, or a Less mixin or detached-ruleset call.
pub(super) fn statement_adds_rules(prelude: &[u8], dialect: Dialect) -> bool {
    let Ok(text) = core::str::from_utf8(prelude) else {
        return true;
    };
    let text = text.trim();
    if let Some((name, rest)) = at_rule(text) {
        return match name {
            "include" | "import" | "forward" | "require" | "plugin" => true,
            // Built-in Sass modules emit no CSS.
            "use" => !rest
                .trim_start()
                .get(1..)
                .is_some_and(|url| url.starts_with("sass:")),
            _ => dialect == Dialect::Less && rest.trim_start().starts_with('('),
        };
    }
    dialect == Dialect::Less && text.starts_with(['.', '#'])
}

/// The name and the rest of an at-rule, `@name rest`.
pub(super) fn at_rule(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix('@')?;
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':')))
        .unwrap_or(rest.len());
    Some((rest.get(..end)?, rest.get(end..)?))
}

pub(super) fn resolve(
    prelude: &[u8],
    parent: Option<Vec<String>>,
    interpolated: bool,
    dialect: Dialect,
) -> Option<Vec<String>> {
    let text = core::str::from_utf8(prelude).ok()?.trim();
    if let Some(at_rule) = text.strip_prefix('@') {
        let (name, rest) = at_rule
            .split_once(|c: char| c.is_ascii_whitespace() || c == '(')
            .unwrap_or((at_rule, ""));
        return match name {
            // Their blocks keep the enclosing selector.
            "media" | "supports" | "container" | "layer" | "include" => parent,
            "at-root" if rest.trim().is_empty() => Some(Vec::new()),
            "at-root" => resolve(rest.as_bytes(), Some(Vec::new()), interpolated, dialect),
            // Mixins, functions and control flow apply elsewhere or repeatedly.
            _ => None,
        };
    }
    let parent = parent?;
    // `font: { family: ... }` nests properties, not selectors.
    if interpolated || dialect == Dialect::Scss && is_nested_property(text) {
        return None;
    }
    let parts = split_top_level_commas(text);
    if dialect == Dialect::Less && parts.iter().any(|part| is_less_mixin_definition(part)) {
        return None;
    }
    let mut resolved = Vec::new();
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if parent.is_empty() {
            if !part.contains('&') {
                resolved.push(String::from(part));
            }
            continue;
        }
        for outer in &parent {
            resolved.push(nest(part, outer));
        }
    }
    (resolved.len() <= MAX_SELECTORS).then_some(resolved)
}

/// `&` names the parent; a selector without one descends from it.
fn nest(part: &str, outer: &str) -> String {
    let mut selector = String::default();
    if part.contains('&') {
        for (index, piece) in part.split('&').enumerate() {
            if index > 0 {
                selector.push_str(outer);
            }
            selector.push_str(piece);
        }
    } else {
        selector.push_str(outer);
        selector.push(' ');
        selector.push_str(part);
    }
    selector
}

fn is_nested_property(text: &str) -> bool {
    let Some((name, value)) = text.split_once(':') else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && (value.is_empty() || value.starts_with(|c: char| c.is_ascii_whitespace()))
}

/// `.name(` or `#name(` opens a Less mixin definition, which emits nothing.
fn is_less_mixin_definition(part: &str) -> bool {
    let part = part.trim_start();
    let Some(rest) = part.strip_prefix(['.', '#']) else {
        return false;
    };
    let name_end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    name_end > 0
        && rest
            .get(name_end..)
            .is_some_and(|tail| tail.trim_start().starts_with('('))
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        match byte {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.extend(text.get(start..index));
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.extend(text.get(start..));
    parts
}
