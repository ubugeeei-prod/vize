//! Scope selectors inside native CSS nesting after the outer rule is scoped.

use super::{leading_css_comment_trivia_end, scope_selector_with_leading_comments};
use vize_carton::String;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Selector,
    AtRule,
    Keyframes,
    Value,
}

struct Block {
    kind: BlockKind,
    segment_start: usize,
}

pub(super) fn scope_nested_selectors(css: &str, attr_selector: &str) -> Option<String> {
    let mut output = None::<String>;
    let mut blocks = Vec::<Block>::new();
    let mut root_segment_start = 0usize;
    let mut copied_through = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut in_comment = false;
    let mut iter = css.char_indices().peekable();

    while let Some((index, ch)) = iter.next() {
        let next = iter.peek().map(|(_, next)| *next);
        if in_comment {
            if ch == '*' && next == Some('/') {
                iter.next();
                in_comment = false;
            }
            continue;
        }
        if let Some(active_quote) = quote {
            if ch == '\\' {
                iter.next();
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '/' if next == Some('*') => {
                iter.next();
                in_comment = true;
            }
            '\\' => {
                iter.next();
            }
            '\'' | '"' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' if paren_depth == 0 && bracket_depth == 0 => {
                let header_start = blocks
                    .last()
                    .map_or(root_segment_start, |block| block.segment_start);
                let header = css.get(header_start..index).unwrap_or_default();
                let kind = classify_header(header);

                if kind == BlockKind::Selector
                    && blocks.iter().any(|block| block.kind == BlockKind::Selector)
                    && !blocks
                        .iter()
                        .any(|block| block.kind == BlockKind::Keyframes)
                {
                    let rewritten = output.get_or_insert_with(|| {
                        String::with_capacity(css.len() + attr_selector.len())
                    });
                    rewritten.push_str(css.get(copied_through..header_start).unwrap_or_default());
                    rewritten.push_str(scope_header(header, attr_selector).as_str());
                    copied_through = index;
                }
                blocks.push(Block {
                    kind,
                    segment_start: index + 1,
                });
            }
            ';' if paren_depth == 0 && bracket_depth == 0 => {
                if let Some(block) = blocks.last_mut() {
                    block.segment_start = index + 1;
                } else {
                    root_segment_start = index + 1;
                }
            }
            '}' if paren_depth == 0 && bracket_depth == 0 => {
                blocks.pop();
                if let Some(block) = blocks.last_mut() {
                    block.segment_start = index + 1;
                } else {
                    root_segment_start = index + 1;
                }
            }
            _ => {}
        }
    }

    output.map(|mut rewritten| {
        rewritten.push_str(css.get(copied_through..).unwrap_or_default());
        rewritten
    })
}

fn classify_header(header: &str) -> BlockKind {
    let body = leading_css_comment_trivia_end(header)
        .and_then(|end| header.get(end..))
        .unwrap_or(header)
        .trim_start();
    if body.starts_with("@keyframes")
        || body.starts_with("@-webkit-keyframes")
        || body.starts_with("@-moz-keyframes")
        || body.starts_with("@-o-keyframes")
    {
        BlockKind::Keyframes
    } else if body.starts_with('@') {
        BlockKind::AtRule
    } else if body.starts_with("--") && body.contains(':') {
        BlockKind::Value
    } else {
        BlockKind::Selector
    }
}

fn scope_header(header: &str, attr_selector: &str) -> String {
    let leading_end = header
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map_or(header.len(), |(index, _)| index);
    let (leading, selector) = header.split_at_checked(leading_end).unwrap_or(("", header));
    let mut output = String::with_capacity(header.len() + attr_selector.len());
    output.push_str(leading);
    output.push_str(scope_selector_with_leading_comments(selector, attr_selector).as_str());
    output
}
