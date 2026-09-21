//! The S1 page printer: the canonical `Full` text.

use core::fmt::{Result, Write};

use super::{PageAttribute, PageClose, PageElement, PageNode, PageToken, SurfacePage};

pub(super) fn print<W: Write>(page: &SurfacePage, w: &mut W) -> Result {
    writeln!(w, "[s1]")?;
    writeln!(w, "bytes={}", page.bytes())?;
    writeln!(w)?;
    if page.children.is_empty() {
        return Ok(());
    }
    writeln!(w, "[s1.tree]")?;
    for node in &page.children {
        node_lines(node, 0, w)?;
    }
    writeln!(w)
}

fn indent<W: Write>(depth: usize, w: &mut W) -> Result {
    for _ in 0..depth {
        w.write_str("  ")?;
    }
    Ok(())
}

pub(super) fn token_line<W: Write>(
    role: &str,
    token: &PageToken,
    depth: usize,
    w: &mut W,
) -> Result {
    indent(depth, w)?;
    write!(w, "{role} {}:{}:{}", token.start, token.text, token.end)?;
    if token.missing {
        w.write_str(" missing")?;
    }
    writeln!(w)
}

fn word_line<W: Write>(word: &str, depth: usize, w: &mut W) -> Result {
    indent(depth, w)?;
    writeln!(w, "{word}")
}

fn node_lines<W: Write>(node: &PageNode, depth: usize, w: &mut W) -> Result {
    match node {
        PageNode::Element(element) => element_lines(element, depth, w),
        PageNode::Interpolation(node) => {
            word_line("interpolation", depth, w)?;
            token_line("open", &node.open, depth + 1, w)?;
            token_line("content", &node.content, depth + 1, w)?;
            token_line("close", &node.close, depth + 1, w)
        }
        PageNode::Text(token) => token_line("text", token, depth, w),
        PageNode::Comment(token) => token_line("comment", token, depth, w),
        PageNode::Cdata(token) => token_line("cdata", token, depth, w),
        PageNode::ProcessingInstruction(token) => token_line("pi", token, depth, w),
        PageNode::Unexpected(token) => token_line("unexpected", token, depth, w),
    }
}

fn element_lines<W: Write>(element: &PageElement, depth: usize, w: &mut W) -> Result {
    word_line("element", depth, w)?;
    let inner = depth + 1;
    token_line("lt-name", &element.lt_name, inner, w)?;
    for attr in &element.attrs {
        attr_lines(attr, inner, w)?;
    }
    if let Some(slash) = &element.slash {
        token_line("slash", slash, inner, w)?;
    }
    token_line("gt", &element.gt, inner, w)?;
    for child in &element.children {
        node_lines(child, inner, w)?;
    }
    match &element.close {
        PageClose::Tag { lt_slash_name, gt } => {
            word_line("close-tag", inner, w)?;
            token_line("lt-slash-name", lt_slash_name, inner + 1, w)?;
            token_line("gt", gt, inner + 1, w)
        }
        PageClose::Implicit => word_line("close implicit", inner, w),
        PageClose::Missing => word_line("close missing", inner, w),
        PageClose::NotExpected => word_line("close not-expected", inner, w),
    }
}

fn attr_lines<W: Write>(attr: &PageAttribute, depth: usize, w: &mut W) -> Result {
    word_line("attr", depth, w)?;
    let inner = depth + 1;
    token_line("name", &attr.name, inner, w)?;
    if let Some(eq) = &attr.eq {
        token_line("eq", eq, inner, w)?;
    }
    if let Some(value) = &attr.value {
        if let Some(open_quote) = &value.open_quote {
            token_line("open-quote", open_quote, inner, w)?;
        }
        token_line("value", &value.content, inner, w)?;
        if let Some(close_quote) = &value.close_quote {
            token_line("close-quote", close_quote, inner, w)?;
        }
    }
    Ok(())
}
