//! Pretty-printer for the [`super::ast`] value tree.

use super::ast::{Comment, Node};
use vize_carton::String;

pub(super) struct Printer<'a> {
    pub(super) indent: &'a str,
    pub(super) newline: &'a str,
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

    pub(super) fn write_value(&self, output: &mut String, node: &Node, depth: usize) {
        match node {
            Node::Scalar(text) => output.push_str(text.as_str()),
            Node::Object { members, dangling } => {
                self.write_block(
                    output,
                    depth,
                    ['{', '}'],
                    members.len(),
                    dangling,
                    |p, out| {
                        for (index, member) in members.iter().enumerate() {
                            p.write_leading(out, &member.leading, depth + 1);
                            out.push_str(p.newline);
                            p.write_indent(out, depth + 1);
                            out.push_str(member.key.as_str());
                            out.push_str(": ");
                            p.write_value(out, &member.value, depth + 1);
                            if index + 1 < members.len() {
                                out.push(',');
                            }
                            p.write_trailing(out, &member.trailing);
                        }
                    },
                );
            }
            Node::Array { elements, dangling } => {
                self.write_block(
                    output,
                    depth,
                    ['[', ']'],
                    elements.len(),
                    dangling,
                    |p, out| {
                        for (index, element) in elements.iter().enumerate() {
                            p.write_leading(out, &element.leading, depth + 1);
                            out.push_str(p.newline);
                            p.write_indent(out, depth + 1);
                            p.write_value(out, &element.value, depth + 1);
                            if index + 1 < elements.len() {
                                out.push(',');
                            }
                            p.write_trailing(out, &element.trailing);
                        }
                    },
                );
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
        for comment in dangling {
            output.push_str(self.newline);
            self.write_indent(output, depth + 1);
            self.write_comment(output, comment);
        }
        output.push_str(self.newline);
        self.write_indent(output, depth);
        output.push(close);
    }

    fn write_leading(&self, output: &mut String, comments: &[Comment], depth: usize) {
        for comment in comments {
            output.push_str(self.newline);
            self.write_indent(output, depth);
            self.write_comment(output, comment);
        }
    }

    fn write_trailing(&self, output: &mut String, comments: &[Comment]) {
        for comment in comments {
            output.push(' ');
            self.write_comment(output, comment);
        }
    }
}
