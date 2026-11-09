//! Authored class names in SCSS and Less, with nested selectors resolved
//! against their parents (`.card { &__title {} }` declares `card__title`).
//!
//! Rules whose selector cannot be known statically (interpolation, mixin and
//! control-flow bodies) contribute nothing; their siblings still do.

use std::collections::BTreeSet;

use vize_carton::String as CompactString;

use super::collect_classes_from_selector_prelude;

/// A resolved selector list grows multiplicatively with nested comma lists.
const MAX_SELECTORS: usize = 256;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Dialect {
    Scss,
    Less,
}

/// Class names SCSS rules declare, or `None` when the source is unbalanced.
pub fn authored_local_scss_classes(scss: &str) -> Option<BTreeSet<CompactString>> {
    authored_local_nested_classes(scss, Dialect::Scss)
}

/// Class names Less rules declare, or `None` when the source is unbalanced.
/// Mixin definitions (`.m() {}`) emit no rules and add no names.
pub fn authored_local_less_classes(less: &str) -> Option<BTreeSet<CompactString>> {
    authored_local_nested_classes(less, Dialect::Less)
}

pub(super) fn authored_local_nested_classes(
    scss: &str,
    dialect: Dialect,
) -> Option<BTreeSet<CompactString>> {
    let bytes = scss.as_bytes();
    let interpolation = match dialect {
        Dialect::Scss => b'#',
        Dialect::Less => b'@',
    };
    let mut classes = BTreeSet::new();
    // The resolved selectors of every open block; `None` is an unknown context.
    let mut blocks: Vec<Option<Vec<CompactString>>> = Vec::new();
    let mut prelude = Vec::new();
    let mut interpolated = false;
    let mut parens = 0usize;
    let mut quote = None;
    let mut index = 0usize;

    while let Some(&byte) = bytes.get(index) {
        if let Some(open) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == open {
                quote = None;
            }
            prelude.push(byte);
            index += 1;
            continue;
        }
        let next = bytes.get(index + 1).copied();
        match (byte, next) {
            (b'/', Some(b'*')) => {
                index += scss.get(index + 2..)?.find("*/")? + 4;
                continue;
            }
            // `//` inside `url(...)` is part of the value.
            (b'/', Some(b'/')) if parens == 0 => {
                index += scss.get(index..)?.find('\n').unwrap_or(bytes.len() - index);
                continue;
            }
            (opener, Some(b'{')) if opener == interpolation => {
                index += scss.get(index..)?.find('}')? + 1;
                interpolated = true;
                prelude.extend_from_slice(b"#");
                continue;
            }
            (b'\'' | b'"', _) => quote = Some(byte),
            (b'(', _) => parens += 1,
            (b')', _) => parens = parens.saturating_sub(1),
            (b'{', _) => {
                let parent = match blocks.last() {
                    Some(parent) => parent.clone(),
                    None => Some(Vec::new()),
                };
                let resolved = resolve(&prelude, parent, interpolated, dialect);
                for selector in resolved.iter().flatten() {
                    // A rule outside the supported selector grammar adds no names.
                    let mut rule = BTreeSet::new();
                    if collect_classes_from_selector_prelude(selector.as_bytes(), &mut rule)
                        .is_some()
                    {
                        classes.extend(rule);
                    }
                }
                blocks.push(resolved);
                prelude.clear();
                interpolated = false;
                parens = 0;
                index += 1;
                continue;
            }
            (b';', _) => {
                prelude.clear();
                interpolated = false;
                parens = 0;
                index += 1;
                continue;
            }
            (b'}', _) => {
                blocks.pop()?;
                prelude.clear();
                interpolated = false;
                parens = 0;
                index += 1;
                continue;
            }
            _ => {}
        }
        prelude.push(byte);
        index += 1;
    }
    (quote.is_none() && blocks.is_empty()).then_some(classes)
}

pub(super) fn resolve(
    prelude: &[u8],
    parent: Option<Vec<CompactString>>,
    interpolated: bool,
    dialect: Dialect,
) -> Option<Vec<CompactString>> {
    let text = std::str::from_utf8(prelude).ok()?.trim();
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
                resolved.push(CompactString::from(part));
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
fn nest(part: &str, outer: &str) -> CompactString {
    let mut selector = CompactString::default();
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

#[cfg(test)]
mod tests {
    use super::{authored_local_less_classes, authored_local_scss_classes};

    fn names(scss: &str) -> Vec<vize_carton::String> {
        authored_local_scss_classes(scss)
            .expect("balanced SCSS")
            .into_iter()
            .collect()
    }

    #[test]
    fn nested_parent_references_resolve_against_every_parent() {
        assert_eq!(
            names(
                ".card { &__title { color: red; } &--active {} .icon {} &:hover {} }\n.a, .b { &-x {} }"
            ),
            [
                "a",
                "a-x",
                "b",
                "b-x",
                "card",
                "card--active",
                "card__title",
                "icon"
            ]
        );
    }

    #[test]
    fn comments_values_and_nested_properties_add_no_names() {
        assert_eq!(
            names(
                "// .line {}\n/* .block {} */\n$gap: 4px;\n@use 'theme';\n.u {\n  background: url(http://e.test/a.png); // .trailing {}\n  font: { family: serif; }\n  content: '.quoted {';\n  &-v {}\n}"
            ),
            ["u", "u-v"]
        );
    }

    #[test]
    fn at_rules_keep_or_drop_the_enclosing_selector() {
        assert_eq!(
            names(
                "@media (max-width: 1px) { .m { &-n {} } }\n.p { @media print { &-q {} } @include wide { &-r {} } @at-root .s {} }\n@mixin m { .mixed { &-x {} } }\n@each $k in a, b { .each-#{$k} {} }\n.i-#{$name} { &-j {} }\n.ok {}\n:global(.g) .l {}"
            ),
            ["l", "m", "m-n", "ok", "p", "p-q", "p-r", "s"]
        );
    }

    #[test]
    fn less_resolves_parents_and_skips_mixin_definitions() {
        let names = authored_local_less_classes(
            "@gap: 4px;\n.card { &__title {} &:extend(.base); .icon {} }\n.m() { .hidden {} }\n.guarded(@a) when (@a > 1) { .alsoHidden {} }\n#ns { .inner() {} }\n.pre-@{name} { &-x {} }\n@media screen { .m2 { &-n {} } }\n@detached: { .det {} };\n// .commented {}\n.call { .m(); }",
        )
        .expect("balanced Less");
        assert_eq!(
            names.into_iter().collect::<Vec<_>>(),
            ["call", "card", "card__title", "icon", "m2", "m2-n"]
        );
    }

    #[test]
    fn unbalanced_sources_refuse() {
        assert_eq!(authored_local_scss_classes(".a { &-b {"), None);
        assert_eq!(authored_local_scss_classes(".a {} }"), None);
    }
}
