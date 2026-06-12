//! JSX text whitespace handling.
//!
//! JSX text is cleaned with the same rule `@vue/babel-plugin-jsx` inherits from
//! Babel: lines are split, tabs become spaces, whitespace adjacent to a newline
//! is trimmed, blank lines are dropped, and the remaining lines are joined with
//! a single space. Leading whitespace on the first line and trailing whitespace
//! on the last line are preserved.

use oxc_ast::ast::JSXText;
use vize_carton::{Box, String};
use vize_relief::ast::{TemplateChildNode, TextNode};

use super::Lowerer;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower a JSX text child, returning `None` if it cleans to nothing.
    pub(crate) fn lower_text(&mut self, text: &JSXText<'_>) -> Option<TemplateChildNode<'a>> {
        let cleaned = clean_jsx_text(text.value.as_str());
        if cleaned.is_empty() {
            return None;
        }
        let loc = self.mapper().location(text.span);
        Some(TemplateChildNode::Text(Box::new_in(
            TextNode::new(cleaned, loc),
            self.bump(),
        )))
    }
}

/// Clean JSX text per the Babel/Vue JSX whitespace algorithm.
pub(crate) fn clean_jsx_text(raw: &str) -> String {
    // Normalize line endings by stripping a trailing `\r` from each `\n`-split
    // line so `\r\n` collapses to a single line break.
    let lines: std::vec::Vec<&str> = raw
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let line_count = lines.len();

    // Index of the last line containing a non-whitespace byte. Babel keeps the
    // initial value of `0` when every line is blank, so we mirror that.
    let last_non_blank = lines
        .iter()
        .rposition(|line| line.bytes().any(|b| b != b' ' && b != b'\t'))
        .unwrap_or(0);

    let mut result = String::default();
    for (index, line) in lines.iter().enumerate() {
        let is_first = index == 0;
        let is_last = index == line_count - 1;

        // Tabs are treated as spaces, matching Babel's `replace(/\t/g, " ")`.
        let normalized = line.replace('\t', " ");
        let mut trimmed: &str = &normalized;
        if !is_first {
            trimmed = trimmed.trim_start_matches(' ');
        }
        if !is_last {
            trimmed = trimmed.trim_end_matches(' ');
        }
        if trimmed.is_empty() {
            continue;
        }
        result.push_str(trimmed);
        if index != last_non_blank {
            result.push(' ');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::clean_jsx_text;

    #[test]
    fn collapses_indentation_between_lines() {
        let input = "\n      Hello\n      World\n    ";
        assert_eq!(clean_jsx_text(input), "Hello World");
    }

    #[test]
    fn preserves_single_line_internal_spaces() {
        assert_eq!(clean_jsx_text("a   b"), "a   b");
    }

    #[test]
    fn preserves_first_line_leading_and_last_line_trailing() {
        assert_eq!(clean_jsx_text("  hi  "), "  hi  ");
    }

    #[test]
    fn whitespace_only_is_empty() {
        assert_eq!(clean_jsx_text("\n   \n   \n"), "");
        assert_eq!(clean_jsx_text("   "), "   ");
    }

    #[test]
    fn crlf_is_normalized() {
        assert_eq!(clean_jsx_text("a\r\n  b"), "a b");
    }
}
