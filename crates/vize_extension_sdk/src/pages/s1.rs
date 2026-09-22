//! The S1 page writer (`s1-page@1`).
//!
//! The page records structure and block-relative offsets only: a token is
//! `start:text:end`, where `start..text` is its verbatim leading gap and
//! `text..end` its own bytes. The host rebuilds the tree from the block it
//! sent and refuses a page whose tokens do not tile the block exactly, so a
//! guest's tokens must cover every byte, in order, once.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

/// One token. `missing` marks a typed hole (zero-width: `text == end`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub start: u32,
    pub text: u32,
    pub end: u32,
    pub missing: bool,
}

impl Token {
    /// A present token: leading gap `start..text`, own bytes `text..end`.
    #[must_use]
    pub const fn new(start: u32, text: u32, end: u32) -> Self {
        Self {
            start,
            text,
            end,
            missing: false,
        }
    }

    /// A typed hole at `at`, after the leading gap `start..at`.
    #[must_use]
    pub const fn missing(start: u32, at: u32) -> Self {
        Self {
            start,
            text: at,
            end: at,
            missing: true,
        }
    }
}

/// A child at any children level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(Token),
    Interpolation {
        open: Token,
        content: Token,
        close: Token,
    },
    Comment(Token),
    Cdata(Token),
    ProcessingInstruction(Token),
    Unexpected(Token),
}

/// An element: open tag, children, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub lt_name: Token,
    pub attrs: Vec<Attribute>,
    pub slash: Option<Token>,
    pub gt: Token,
    pub children: Vec<Node>,
    pub close: Close,
}

/// An attribute: name, optional `=`, optional value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: Token,
    pub eq: Option<Token>,
    pub value: Option<AttrValue>,
}

/// An attribute value with its optional quotes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrValue {
    pub open_quote: Option<Token>,
    pub content: Token,
    pub close_quote: Option<Token>,
}

/// How an element ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Close {
    Tag { lt_slash_name: Token, gt: Token },
    Implicit,
    Missing,
    NotExpected,
}

/// A whole S1 page: the root children level.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfacePage {
    pub children: Vec<Node>,
}

impl SurfacePage {
    /// The canonical page text.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        let mut end = 0;
        last_end(&self.children, &mut end);
        let _ = writeln!(out, "[s1]\nbytes={end}\n");
        if !self.children.is_empty() {
            out.push_str("[s1.tree]\n");
            for node in &self.children {
                node_lines(&mut out, node, 0);
            }
            out.push('\n');
        }
        out
    }
}

fn last_end(nodes: &[Node], end: &mut u32) {
    for node in nodes {
        *end = match node {
            Node::Element(element) => {
                last_end(&element.children, end);
                match &element.close {
                    Close::Tag { gt, .. } => gt.end,
                    _ if element.children.is_empty() => element.gt.end,
                    _ => *end,
                }
            }
            Node::Interpolation { close, .. } => close.end,
            Node::Text(token)
            | Node::Comment(token)
            | Node::Cdata(token)
            | Node::ProcessingInstruction(token)
            | Node::Unexpected(token) => token.end,
        };
    }
}

fn line(out: &mut String, depth: usize, word: &str) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(word);
    out.push('\n');
}

fn token(out: &mut String, depth: usize, role: &str, token: &Token) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    let _ = write!(out, "{role} {}:{}:{}", token.start, token.text, token.end);
    if token.missing {
        out.push_str(" missing");
    }
    out.push('\n');
}

fn node_lines(out: &mut String, node: &Node, depth: usize) {
    match node {
        Node::Element(element) => {
            line(out, depth, "element");
            let inner = depth + 1;
            token(out, inner, "lt-name", &element.lt_name);
            for attr in &element.attrs {
                line(out, inner, "attr");
                token(out, inner + 1, "name", &attr.name);
                if let Some(eq) = &attr.eq {
                    token(out, inner + 1, "eq", eq);
                }
                if let Some(value) = &attr.value {
                    if let Some(open) = &value.open_quote {
                        token(out, inner + 1, "open-quote", open);
                    }
                    token(out, inner + 1, "value", &value.content);
                    if let Some(close) = &value.close_quote {
                        token(out, inner + 1, "close-quote", close);
                    }
                }
            }
            if let Some(slash) = &element.slash {
                token(out, inner, "slash", slash);
            }
            token(out, inner, "gt", &element.gt);
            for child in &element.children {
                node_lines(out, child, inner);
            }
            match &element.close {
                Close::Tag { lt_slash_name, gt } => {
                    line(out, inner, "close-tag");
                    token(out, inner + 1, "lt-slash-name", lt_slash_name);
                    token(out, inner + 1, "gt", gt);
                }
                Close::Implicit => line(out, inner, "close implicit"),
                Close::Missing => line(out, inner, "close missing"),
                Close::NotExpected => line(out, inner, "close not-expected"),
            }
        }
        Node::Interpolation {
            open,
            content,
            close,
        } => {
            line(out, depth, "interpolation");
            token(out, depth + 1, "open", open);
            token(out, depth + 1, "content", content);
            token(out, depth + 1, "close", close);
        }
        Node::Text(t) => token(out, depth, "text", t),
        Node::Comment(t) => token(out, depth, "comment", t),
        Node::Cdata(t) => token(out, depth, "cdata", t),
        Node::ProcessingInstruction(t) => token(out, depth, "pi", t),
        Node::Unexpected(t) => token(out, depth, "unexpected", t),
    }
}
