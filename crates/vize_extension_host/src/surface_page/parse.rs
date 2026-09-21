//! The S1 page parser.
//!
//! Lenient exactly where the printer normalizes — blank-line placement
//! (separators vanish), the `bytes=` statement (validated as an integer,
//! recomputed by print) and non-canonical integer spellings — and strict
//! everywhere else: the section order, the fixed grammar under an element
//! and an attribute, and two-space indentation. Offsets are only checked
//! for local order here (`start <= text <= end`, a `missing` token is
//! zero-width); tiling against a source is [`SurfacePage::check_tiles`].
//!
//! [`SurfacePage::check_tiles`]: super::SurfacePage::check_tiles

use vize_davinci::folio::FolioError;
use vize_s0::{String, cstr};

use super::{
    PageAttrValue, PageAttribute, PageClose, PageElement, PageInterpolation, PageNode, PageToken,
    SurfacePage,
};

struct Line<'a> {
    no: usize,
    depth: usize,
    text: &'a str,
}

struct Cursor<'a> {
    lines: Vec<Line<'a>>,
    at: usize,
}

fn err(line: usize, message: String) -> FolioError {
    FolioError::new(line, message)
}

pub(super) fn parse(input: &str) -> Result<SurfacePage, FolioError> {
    let mut section = "";
    let mut seen_bytes = false;
    let mut seen_tree = false;
    let mut lines = Vec::new();
    for (index, line) in input.split('\n').enumerate() {
        let no = index + 1;
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = match (section, name) {
                ("", "s1") => "s1",
                ("", _) => return Err(err(no, cstr!("first section must be [s1]"))),
                (_, "s1") => return Err(err(no, cstr!("duplicate section [s1]"))),
                (_, "s1.tree") if seen_tree => {
                    return Err(err(no, cstr!("duplicate section [s1.tree]")));
                }
                (_, "s1.tree") => {
                    seen_tree = true;
                    "s1.tree"
                }
                (_, other) => return Err(err(no, cstr!("unknown section [{other}]"))),
            };
            continue;
        }
        if line.is_empty() {
            continue;
        }
        match section {
            "" => return Err(err(no, cstr!("content before the [s1] header"))),
            "s1" => {
                let Some(value) = line.strip_prefix("bytes=") else {
                    return Err(err(no, cstr!("unknown field line `{line}`")));
                };
                if seen_bytes {
                    return Err(err(no, cstr!("duplicate field `bytes`")));
                }
                if value.parse::<u32>().is_err() {
                    return Err(err(no, cstr!("invalid integer `{value}`")));
                }
                seen_bytes = true;
            }
            _ => {
                let text = line.trim_start_matches(' ');
                let spaces = line.len() - text.len();
                if !spaces.is_multiple_of(2) {
                    return Err(err(no, cstr!("odd indentation")));
                }
                let depth = spaces / 2;
                lines.push(Line { no, depth, text });
            }
        }
    }
    if !seen_bytes {
        return Err(err(0, cstr!("missing field `bytes`")));
    }
    let mut cursor = Cursor { lines, at: 0 };
    let children = cursor.children(0)?;
    if let Some(line) = cursor.lines.get(cursor.at) {
        let message = if line.depth > 0 {
            cstr!("over-indented line")
        } else {
            cstr!("expected a node")
        };
        return Err(err(line.no, message));
    }
    Ok(SurfacePage { children })
}

const NODE_WORDS: &[&str] = &[
    "element",
    "interpolation",
    "text",
    "comment",
    "cdata",
    "pi",
    "unexpected",
];

impl Cursor<'_> {
    /// The first word of the next line when it sits at exactly `depth`.
    fn peek(&self, depth: usize) -> Option<&str> {
        let line = self.lines.get(self.at)?;
        (line.depth == depth).then(|| line.text.split(' ').next().unwrap_or(""))
    }

    fn eof(role: &str) -> FolioError {
        err(0, cstr!("unexpected end of input: expected `{role}`"))
    }

    fn children(&mut self, depth: usize) -> Result<Vec<PageNode>, FolioError> {
        let mut nodes = Vec::new();
        while let Some(word) = self.peek(depth) {
            if !NODE_WORDS.contains(&word) {
                break;
            }
            nodes.push(self.node(depth)?);
        }
        Ok(nodes)
    }

    fn node(&mut self, depth: usize) -> Result<PageNode, FolioError> {
        let line = &self.lines[self.at];
        let (no, text) = (line.no, line.text);
        match text {
            "element" => {
                self.at += 1;
                return self.element(depth + 1).map(PageNode::Element);
            }
            "interpolation" => {
                self.at += 1;
                return Ok(PageNode::Interpolation(PageInterpolation {
                    open: self.token(depth + 1, "open")?,
                    content: self.token(depth + 1, "content")?,
                    close: self.token(depth + 1, "close")?,
                }));
            }
            _ => {}
        }
        let (role, rest) = text.split_once(' ').unwrap_or((text, ""));
        let leaf: fn(PageToken) -> PageNode = match role {
            "text" => PageNode::Text,
            "comment" => PageNode::Comment,
            "cdata" => PageNode::Cdata,
            "pi" => PageNode::ProcessingInstruction,
            "unexpected" => PageNode::Unexpected,
            _ => return Err(err(no, cstr!("`{role}` takes no operands"))),
        };
        let token = parse_token(no, rest)?;
        self.at += 1;
        Ok(leaf(token))
    }

    fn token(&mut self, depth: usize, role: &str) -> Result<PageToken, FolioError> {
        let Some(line) = self.lines.get(self.at) else {
            return Err(Self::eof(role));
        };
        let (found, rest) = line.text.split_once(' ').unwrap_or((line.text, ""));
        if line.depth != depth || found != role {
            return Err(err(line.no, cstr!("expected `{role}`")));
        }
        let token = parse_token(line.no, rest)?;
        self.at += 1;
        Ok(token)
    }

    fn optional(&mut self, depth: usize, role: &str) -> Result<Option<PageToken>, FolioError> {
        if self.peek(depth) == Some(role) {
            return self.token(depth, role).map(Some);
        }
        Ok(None)
    }

    fn element(&mut self, depth: usize) -> Result<PageElement, FolioError> {
        let lt_name = self.token(depth, "lt-name")?;
        let mut attrs = Vec::new();
        while self.peek(depth) == Some("attr") && self.lines[self.at].text == "attr" {
            self.at += 1;
            attrs.push(self.attribute(depth + 1)?);
        }
        let slash = self.optional(depth, "slash")?;
        let gt = self.token(depth, "gt")?;
        let children = self.children(depth)?;
        let Some(line) = self.lines.get(self.at) else {
            return Err(Self::eof("close"));
        };
        let close = match (line.depth == depth, line.text) {
            (true, "close-tag") => {
                self.at += 1;
                let lt_slash_name = self.token(depth + 1, "lt-slash-name")?;
                let gt = self.token(depth + 1, "gt")?;
                PageClose::Tag { lt_slash_name, gt }
            }
            (true, "close implicit") => PageClose::Implicit,
            (true, "close missing") => PageClose::Missing,
            (true, "close not-expected") => PageClose::NotExpected,
            _ => return Err(err(line.no, cstr!("expected an element child or `close`"))),
        };
        if !matches!(close, PageClose::Tag { .. }) {
            self.at += 1;
        }
        Ok(PageElement {
            lt_name,
            attrs,
            slash,
            gt,
            children,
            close,
        })
    }

    fn attribute(&mut self, depth: usize) -> Result<PageAttribute, FolioError> {
        let name = self.token(depth, "name")?;
        let eq = self.optional(depth, "eq")?;
        let open_quote = self.optional(depth, "open-quote")?;
        let value = if open_quote.is_some() || self.peek(depth) == Some("value") {
            let content = self.token(depth, "value")?;
            let close_quote = self.optional(depth, "close-quote")?;
            Some(PageAttrValue {
                open_quote,
                content,
                close_quote,
            })
        } else {
            None
        };
        Ok(PageAttribute { name, eq, value })
    }
}

fn parse_token(no: usize, rest: &str) -> Result<PageToken, FolioError> {
    let (offsets, missing) = match rest.split_once(' ') {
        None => (rest, false),
        Some((offsets, "missing")) => (offsets, true),
        Some((_, other)) => return Err(err(no, cstr!("unknown token flag `{other}`"))),
    };
    let mut parts = offsets.split(':');
    let mut next = || -> Result<u32, FolioError> {
        let part = parts.next().unwrap_or("");
        part.parse::<u32>()
            .map_err(|_| err(no, cstr!("invalid offset `{part}` in `{offsets}`")))
    };
    let (start, text, end) = (next()?, next()?, next()?);
    if parts.next().is_some() {
        return Err(err(no, cstr!("a token is exactly `start:text:end`")));
    }
    if start > text || text > end {
        return Err(err(no, cstr!("token offsets out of order in `{offsets}`")));
    }
    if missing && text != end {
        return Err(err(no, cstr!("a missing token is zero-width")));
    }
    Ok(PageToken {
        start,
        text,
        end,
        missing,
    })
}
