//! The S2 page writer (`s2-page@1`, the disegno grammar) for the op subset a
//! template dialect starts with: HTML elements, static attributes and text.
//!
//! Spans are file-absolute (the block's `base` plus block offsets). Values
//! may contain any character; `\\`, `"`, newline, carriage return and tab are
//! escaped, and other control characters are outside the page contract.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::types::Span;

/// A static attribute: `name`, or `name="value"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub name: String,
    pub value: Option<String>,
    pub span: Span,
}

/// One op of the subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// `ui.element` in the HTML namespace.
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<Op>,
        span: Span,
    },
    /// `ui.text`.
    Text { value: String, span: Span },
}

/// A whole S2 page: the root region's ops.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticPage {
    pub ops: Vec<Op>,
}

impl SemanticPage {
    /// The canonical page text.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "[disegno]\nops={}\n", count(&self.ops));
        if !self.ops.is_empty() {
            out.push_str("[disegno.ops]\n");
            for op in &self.ops {
                op_lines(&mut out, op, 0);
            }
            out.push('\n');
        }
        out
    }
}

fn count(ops: &[Op]) -> u64 {
    ops.iter()
        .map(|op| match op {
            Op::Element { children, .. } => 1 + count(children),
            Op::Text { .. } => 1,
        })
        .sum()
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn quoted(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
}

fn span(out: &mut String, span: Span) {
    let _ = writeln!(out, " @{}:{}", span.start, span.end);
}

fn op_lines(out: &mut String, op: &Op, depth: usize) {
    indent(out, depth);
    match op {
        Op::Element {
            tag,
            attrs,
            children,
            span: at,
        } => {
            let _ = write!(out, "ui.element {tag}");
            span(out, *at);
            for attr in attrs {
                indent(out, depth + 1);
                let _ = write!(out, "attr {}", attr.name);
                if let Some(value) = &attr.value {
                    out.push('=');
                    quoted(out, value);
                }
                span(out, attr.span);
            }
            for child in children {
                op_lines(out, child, depth + 1);
            }
        }
        Op::Text { value, span: at } => {
            out.push_str("ui.text ");
            quoted(out, value);
            span(out, *at);
        }
    }
}
