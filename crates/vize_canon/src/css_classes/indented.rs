//! Authored class names in the indented syntaxes (`lang="sass"`, `"stylus"`),
//! where a selector opens a block by indenting the lines after it.

use std::collections::BTreeSet;

use vize_carton::String as CompactString;

use super::collect_classes_from_selector_prelude;
use super::nested::{Dialect, authored_local_nested_classes, resolve};

#[derive(Clone, Copy, PartialEq)]
enum Syntax {
    Sass,
    Stylus,
}

/// Class names indented Sass rules declare.
pub fn authored_local_sass_classes(sass: &str) -> Option<BTreeSet<CompactString>> {
    authored_local_indented_classes(sass, Syntax::Sass)
}

/// Class names Stylus rules declare. Stylus written with braces reads like SCSS.
pub fn authored_local_stylus_classes(stylus: &str) -> Option<BTreeSet<CompactString>> {
    if stylus.lines().any(|line| line.trim_end().ends_with('{')) {
        return authored_local_nested_classes(stylus, Dialect::Scss);
    }
    authored_local_indented_classes(stylus, Syntax::Stylus)
}

fn authored_local_indented_classes(
    source: &str,
    syntax: Syntax,
) -> Option<BTreeSet<CompactString>> {
    let lines = code_lines(source)?;
    let mut classes = BTreeSet::new();
    // Open blocks: their indentation and resolved selectors (`None` is unknown).
    let mut blocks: Vec<(usize, Option<Vec<CompactString>>)> = Vec::new();
    let mut index = 0usize;
    while let Some(&(indent, ref first)) = lines.get(index) {
        // A selector list may continue over lines that end with `,`.
        let mut text = CompactString::from(first.as_str());
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
        if !opens_block {
            continue;
        }
        let parent = match blocks.last() {
            Some((_, parent)) => parent.clone(),
            None => Some(Vec::new()),
        };
        let resolved = selector(&text, syntax).and_then(|(selector, interpolated)| {
            resolve(selector.as_bytes(), parent, interpolated, Dialect::Scss)
        });
        for selector in resolved.iter().flatten() {
            let mut rule = BTreeSet::new();
            if collect_classes_from_selector_prelude(selector.as_bytes(), &mut rule).is_some() {
                classes.extend(rule);
            }
        }
        blocks.push((indent, resolved));
    }
    Some(classes)
}

/// The selector a block-opening line spells, with whether it interpolates, or
/// `None` for mixin, function and control-flow blocks.
fn selector(text: &str, syntax: Syntax) -> Option<(&str, bool)> {
    match syntax {
        Syntax::Sass => {
            if text.starts_with('=') {
                return None;
            }
            // `+mixin` content blocks keep the enclosing selector.
            if text.starts_with('+') {
                return Some(("@include", false));
            }
            Some((text, text.contains("#{")))
        }
        Syntax::Stylus => {
            let keyword = text.split_ascii_whitespace().next().unwrap_or_default();
            let starts_call = text
                .find('(')
                .and_then(|open| text.get(..open))
                .is_some_and(|name| {
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                });
            if starts_call
                || matches!(keyword, "if" | "unless" | "for" | "else")
                || text.contains(['^', '~'])
                || text.contains("../")
            {
                return None;
            }
            Some((text, text.contains('{')))
        }
    }
}

/// Non-empty lines with their indentation, comments removed.
fn code_lines(source: &str) -> Option<Vec<(usize, CompactString)>> {
    let mut lines = Vec::new();
    let mut in_block_comment = false;
    for line in source.lines() {
        let mut code = CompactString::default();
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
        lines.push((indent, CompactString::from(trimmed.trim_start())));
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

#[cfg(test)]
mod tests {
    use super::{authored_local_sass_classes, authored_local_stylus_classes};

    fn names(
        found: Option<std::collections::BTreeSet<vize_carton::String>>,
    ) -> Vec<vize_carton::String> {
        found.expect("readable source").into_iter().collect()
    }

    #[test]
    fn sass_resolves_indented_parents_and_skips_mixins() {
        let sass = ".card\n  &__title\n    color: red\n  // .commented\n  .icon\n    color: blue\n=mixin\n  .hidden\n    color: red\n.a, .b\n  &-x\n    color: red\n@media screen\n  .m\n    &-n\n      color: red\n.i-#{$x}\n  &-j\n    color: red\n.np\n  font:\n    family: serif\n  &-z\n    background: url(http://e.test/a.png)\n/* .block\n   comment */\n+wide\n  .content\n    color: red\n";
        assert_eq!(
            names(authored_local_sass_classes(sass)),
            [
                "a",
                "a-x",
                "b",
                "b-x",
                "card",
                "card__title",
                "content",
                "icon",
                "m",
                "m-n",
                "np",
                "np-z"
            ]
        );
    }

    #[test]
    fn stylus_resolves_indented_parents_and_skips_functions_and_loops() {
        let stylus = ".card\n  &__title\n    color red\n  .icon\n    color blue\nborder-radius(n)\n  -webkit-border-radius n\n.list\n  for i in 1..3\n    .item-{i}\n      width 10px\n  &-x\n    color red\n.a,\n.b\n  &-y\n    color red\n";
        assert_eq!(
            names(authored_local_stylus_classes(stylus)),
            [
                "a",
                "a-y",
                "b",
                "b-y",
                "card",
                "card__title",
                "icon",
                "list",
                "list-x"
            ]
        );
        assert_eq!(
            names(authored_local_stylus_classes(
                ".brace {\n  &-x {\n    color red\n  }\n}\n"
            )),
            ["brace", "brace-x"]
        );
    }
}
