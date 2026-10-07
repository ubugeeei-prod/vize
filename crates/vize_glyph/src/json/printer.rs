//! Pretty-printer for the [`super::ast`] value tree.

use super::ast::{Comment, Node};
use unicode_width::UnicodeWidthStr;
use vize_l0::String;

pub(super) struct Printer<'a> {
    pub(super) indent: &'a str,
    pub(super) newline: &'a str,
    pub(super) print_width: usize,
    pub(super) tab_width: usize,
    pub(super) bracket_spacing: bool,
}

impl Printer<'_> {
    fn write_indent(&self, output: &mut String, depth: usize) {
        for _ in 0..depth {
            output.push_str(self.indent);
        }
    }

    pub(super) fn write_comment(&self, output: &mut String, comment: &Comment) {
        if comment.block {
            output.push_str("/*");
            output.push_str(comment.text.as_str());
            output.push_str("*/");
        } else {
            output.push_str("//");
            output.push_str(comment.text.as_str());
        }
    }

    pub(super) fn write_value(
        &self,
        output: &mut String,
        node: &Node,
        depth: usize,
        suffix_width: usize,
    ) {
        if self.flat_width(node).is_some_and(|width| {
            self.current_column(output) + width + suffix_width <= self.print_width
        }) {
            self.write_flat(output, node);
            return;
        }
        match node {
            Node::Scalar(text) => output.push_str(text.as_str()),
            Node::Object {
                members, dangling, ..
            } => {
                self.write_block(
                    output,
                    depth,
                    ['{', '}'],
                    members.len(),
                    dangling,
                    |p, out| {
                        for (index, member) in members.iter().enumerate() {
                            p.write_leading(out, &member.leading, depth + 1, index > 0);
                            if member.blank_line_before && (index > 0 || !member.leading.is_empty())
                            {
                                out.push_str(p.newline);
                            }
                            out.push_str(p.newline);
                            p.write_indent(out, depth + 1);
                            out.push_str(member.key.as_str());
                            out.push_str(": ");
                            let comma = usize::from(index + 1 < members.len());
                            p.write_value(
                                out,
                                &member.value,
                                depth + 1,
                                comma + p.trailing_width(&member.trailing),
                            );
                            if comma != 0 {
                                out.push(',');
                            }
                            p.write_trailing(out, &member.trailing);
                        }
                    },
                );
            }
            Node::Array { elements, dangling } => {
                let fill = elements.iter().all(|element| {
                    element.leading.is_empty()
                        && element.trailing.is_empty()
                        && matches!(&element.value, Node::Scalar(text) if text.starts_with('-') || text.starts_with(|c: char| c.is_ascii_digit()))
                }) && dangling.is_empty();
                self.write_block(
                    output,
                    depth,
                    ['[', ']'],
                    elements.len(),
                    dangling,
                    |p, out| {
                        for (index, element) in elements.iter().enumerate() {
                            p.write_leading(out, &element.leading, depth + 1, index > 0);
                            if element.blank_line_before
                                && (index > 0 || !element.leading.is_empty())
                            {
                                out.push_str(p.newline);
                            }
                            let comma = usize::from(index + 1 < elements.len());
                            if fill
                                && index > 0
                                && !element.blank_line_before
                                && p.flat_width(&element.value).is_some_and(|width| {
                                    p.current_column(out) + 1 + width + comma <= p.print_width
                                })
                            {
                                out.push(' ');
                            } else {
                                out.push_str(p.newline);
                                p.write_indent(out, depth + 1);
                            }
                            p.write_value(
                                out,
                                &element.value,
                                depth + 1,
                                comma + p.trailing_width(&element.trailing),
                            );
                            if comma != 0 {
                                out.push(',');
                            }
                            p.write_trailing(out, &element.trailing);
                        }
                    },
                );
            }
        }
    }

    fn current_column(&self, output: &str) -> usize {
        output
            .rsplit(['\r', '\n'])
            .next()
            .unwrap_or_default()
            .split('\t')
            .enumerate()
            .map(|(index, text)| text.width() + usize::from(index > 0) * self.tab_width)
            .sum()
    }

    /// A forced break anywhere in a collection prevents its enclosing group
    /// from flattening. Measure without allocating a speculative output string.
    fn flat_width(&self, node: &Node) -> Option<usize> {
        match node {
            Node::Scalar(text) => Some(text.as_str().width()),
            Node::Object {
                members,
                dangling,
                expanded,
            } => {
                if !dangling.is_empty() || (*expanded && !members.is_empty()) {
                    return None;
                }
                let mut width = 2 + usize::from(self.bracket_spacing && !members.is_empty()) * 2;
                for (index, member) in members.iter().enumerate() {
                    if !member.leading.is_empty()
                        || !member.trailing.is_empty()
                        || member.blank_line_before
                    {
                        return None;
                    }
                    width += usize::from(index > 0) * 2
                        + member.key.as_str().width()
                        + 2
                        + self.flat_width(&member.value)?;
                }
                Some(width)
            }
            Node::Array { elements, dangling } => {
                if !dangling.is_empty() {
                    return None;
                }
                let mut width = 2;
                for (index, element) in elements.iter().enumerate() {
                    if !element.leading.is_empty()
                        || !element.trailing.is_empty()
                        || element.blank_line_before
                    {
                        return None;
                    }
                    width += usize::from(index > 0) * 2 + self.flat_width(&element.value)?;
                }
                Some(width)
            }
        }
    }

    fn write_flat(&self, output: &mut String, node: &Node) {
        match node {
            Node::Scalar(text) => output.push_str(text.as_str()),
            Node::Object { members, .. } => {
                output.push('{');
                if self.bracket_spacing && !members.is_empty() {
                    output.push(' ');
                }
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&member.key);
                    output.push_str(": ");
                    self.write_flat(output, &member.value);
                }
                if self.bracket_spacing && !members.is_empty() {
                    output.push(' ');
                }
                output.push('}');
            }
            Node::Array { elements, .. } => {
                output.push('[');
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        output.push_str(", ");
                    }
                    self.write_flat(output, &element.value);
                }
                output.push(']');
            }
        }
    }

    fn write_block(
        &self,
        output: &mut String,
        depth: usize,
        delims: [char; 2],
        item_count: usize,
        dangling: &[Comment],
        write_items: impl FnOnce(&Self, &mut String),
    ) {
        let [open, close] = delims;
        if item_count == 0 && dangling.is_empty() {
            output.push(open);
            output.push(close);
            return;
        }
        output.push(open);
        write_items(self, output);
        self.write_leading(output, dangling, depth + 1, item_count > 0);
        output.push_str(self.newline);
        self.write_indent(output, depth);
        output.push(close);
    }

    fn write_leading(
        &self,
        output: &mut String,
        comments: &[Comment],
        depth: usize,
        allow_first_blank: bool,
    ) {
        for (index, comment) in comments.iter().enumerate() {
            if comment.blank_line_before && (index > 0 || allow_first_blank) {
                output.push_str(self.newline);
            }
            output.push_str(self.newline);
            self.write_indent(output, depth);
            self.write_comment(output, comment);
        }
    }

    fn trailing_width(&self, comments: &[Comment]) -> usize {
        let mut width = 0;
        for comment in comments {
            if !comment.block {
                break;
            }
            let first_line = comment.text.split(['\r', '\n']).next().unwrap_or_default();
            width += 3 + first_line.width(); // space + opening comment marker
            if first_line.len() != comment.text.len() {
                break;
            }
            width += 2; // closing marker on the same line
        }
        width
    }

    fn write_trailing(&self, output: &mut String, comments: &[Comment]) {
        for comment in comments {
            output.push(' ');
            self.write_comment(output, comment);
        }
    }
}
