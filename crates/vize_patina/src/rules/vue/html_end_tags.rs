//! vue/html-end-tags
//!
//! Require an end tag on HTML elements whose end tags the template parser
//! accepts omitting (`li`, `p`, `td`, …). Elements that cannot omit an end
//! tag are already `parser/template` `MissingEndTag` diagnostics, so this
//! rule does not report them again.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <ul><li>a</ul>
//! <p>hello
//! ```
//!
//! ### Valid
//! ```vue
//! <ul><li>a</li></ul>
//! <p>hello</p>
//! <br>
//! <div></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::cstr;
use vize_relief::{ElementNode, ElementType, RootNode, SourceLocation, TemplateChildNode};

use super::authored_closing;

static META: RuleMeta = RuleMeta {
    name: "vue/html-end-tags",
    description: "Require end tags on HTML elements that may omit them",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// HTML optional end tags. Matches the tags `vize_armature` closes without
/// emitting `MissingEndTag`.
const OPTIONAL_END_TAGS: &[&str] = &[
    "li", "dt", "dd", "p", "rt", "rp", "optgroup", "option", "thead", "tbody", "tfoot", "tr", "td",
    "th",
];

/// Start tags that end an open `<p>` in vue-eslint-parser (Vue 2's
/// non-phrasing list), including the ones this tree keeps inside it.
const CLOSES_PARAGRAPH: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "blockquote",
    "body",
    "caption",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "legend",
    "li",
    "menuitem",
    "meta",
    "optgroup",
    "option",
    "param",
    "rp",
    "rt",
    "source",
    "style",
    "summary",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
];

#[derive(Default)]
/// Implements the `vue/html-end-tags` lint rule.
pub struct HtmlEndTags;

struct Extent {
    end: u32,
    reliable: bool,
}

impl Rule for HtmlEndTags {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let limit = u32::try_from(ctx.source.len()).unwrap_or(u32::MAX);
        let _ = walk_children(ctx, &root.children, 0, limit);
    }
}

fn walk_children(
    ctx: &mut LintContext<'_>,
    children: &[TemplateChildNode<'_>],
    mut cursor: u32,
    limit: u32,
) -> Extent {
    let mut reliable = true;
    for (index, child) in children.iter().enumerate() {
        let start = child.loc().span.start;
        if start < cursor {
            return Extent {
                end: limit,
                reliable: false,
            };
        }
        let next_limit = children
            .get(index + 1)
            .map(|next| next.loc().span.start)
            .filter(|start| *start <= limit)
            .unwrap_or(limit);
        if next_limit < start {
            return Extent {
                end: limit,
                reliable: false,
            };
        }
        let extent = child_extent(ctx, child, cursor, next_limit);
        reliable &= extent.reliable;
        cursor = extent.end;
        if !reliable {
            return Extent {
                end: cursor.min(limit),
                reliable: false,
            };
        }
    }
    if cursor > limit {
        Extent {
            end: limit,
            reliable: false,
        }
    } else {
        Extent {
            end: cursor,
            reliable,
        }
    }
}

fn child_extent(
    ctx: &mut LintContext<'_>,
    child: &TemplateChildNode<'_>,
    cursor: u32,
    limit: u32,
) -> Extent {
    match child {
        TemplateChildNode::Element(element) => vize_l0::recursion::ensure_sufficient_stack(|| {
            element_extent(ctx, element, cursor, limit)
        }),
        other => plain_extent(other.loc(), limit),
    }
}

fn plain_extent(loc: &SourceLocation, limit: u32) -> Extent {
    if loc.span.start > limit || loc.span.end > limit {
        Extent {
            end: limit,
            reliable: false,
        }
    } else {
        Extent {
            end: loc.span.end,
            reliable: true,
        }
    }
}

fn element_extent(
    ctx: &mut LintContext<'_>,
    element: &ElementNode<'_>,
    cursor: u32,
    limit: u32,
) -> Extent {
    // Implicit `tbody`/`tr` nodes are zero-width and sit on the following
    // tag's name (`<tr` → `t`). Walking from that point starts past `<tr`.
    if !authored_open(ctx.source, element) {
        return walk_children(ctx, &element.children, cursor, limit);
    }
    let open_end = element.loc.span.end;
    if open_end > limit {
        return Extent {
            end: limit,
            reliable: false,
        };
    }
    if element.is_self_closing
        || authored_self_closing(ctx.source, element)
        || (element.tag_type == ElementType::Element && is_void_element(element.tag))
    {
        return Extent {
            end: open_end,
            reliable: true,
        };
    }
    let inner = walk_children(ctx, &element.children, open_end, limit);
    if !inner.reliable {
        return inner;
    }
    let closer = paragraph_closer(ctx.source, element, inner.end);
    if let Some(at) = closer {
        report_missing(ctx, element, at);
    }
    if let Some(end) = closing_end(ctx.source, inner.end, limit, element.tag) {
        return Extent {
            end,
            reliable: true,
        };
    }
    if closer.is_none() && can_omit_end_tag(element.tag) {
        // Parser recovery may auto-close `<p>` before a custom component even
        // when the source later carries a real `</p>`. Trust authored evidence.
        if authored_closing::find(ctx.source, element.loc.span.end, element.tag).is_none() {
            report_missing(ctx, element, inner.end);
        }
    }
    Extent {
        end: inner.end,
        reliable: true,
    }
}

fn report_missing(ctx: &mut LintContext<'_>, element: &ElementNode<'_>, insert_at: u32) {
    let tag = element.tag;
    let message = cstr!("'<{tag}>' should have end tag.");
    let help = cstr!("Add a closing `</{tag}>` tag.");
    let edit = TextEdit::insert(insert_at, cstr!("</{tag}>"));
    ctx.report(
        LintDiagnostic::warn(
            META.name,
            message,
            element.loc.span.start,
            element.loc.span.end,
        )
        .with_help(help)
        .with_fix(Fix::new("Add the end tag", edit)),
    );
}

/// Where a start tag ends this `<p>` for vue-eslint-parser: a direct child
/// this tree keeps inside the paragraph, or the start tag (another `<p>`
/// included) at which the tree already ended it, even with a later `</p>`.
fn paragraph_closer(source: &str, element: &ElementNode<'_>, inner_end: u32) -> Option<u32> {
    if element.tag != "p" || element.tag_type != ElementType::Element {
        return None;
    }
    let child = element.children.iter().find_map(|child| match child {
        TemplateChildNode::Element(child)
            if CLOSES_PARAGRAPH.contains(&child.tag) && authored_open(source, child) =>
        {
            Some(child.loc.span.start)
        }
        _ => None,
    });
    child.or_else(|| {
        // A child the tree left open was the current node, not the `<p>`.
        if let Some(TemplateChildNode::Element(last)) = element.children.last()
            && !last.is_self_closing
            && !is_void_element(last.tag)
            && !closes_at(source, inner_end, last.tag)
        {
            return None;
        }
        let rest = source.get(inner_end as usize..)?.strip_prefix('<')?;
        let name_len = rest
            .bytes()
            .take_while(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'/' | b'>'))
            .count();
        let name = rest.get(..name_len)?;
        (name == "p" || CLOSES_PARAGRAPH.contains(&name)).then_some(inner_end)
    })
}

/// Whether `</tag>` ends right at `end`.
fn closes_at(source: &str, end: u32, tag: &str) -> bool {
    let Some(before) = source.get(..end as usize) else {
        return false;
    };
    before.strip_suffix('>').is_some_and(|before| {
        before.rfind("</").is_some_and(|open| {
            before
                .get(open + 2..)
                .is_some_and(|name| name.trim_end().eq_ignore_ascii_case(tag))
        })
    })
}

fn authored_open(source: &str, element: &ElementNode<'_>) -> bool {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    if start >= end {
        return false;
    }
    let Some(bytes) = source.as_bytes().get(start..end) else {
        return false;
    };
    if bytes.first() != Some(&b'<') {
        return false;
    }
    let tag = element.tag.as_bytes();
    let Some(name) = bytes.get(1..) else {
        return false;
    };
    if !name
        .get(..tag.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(tag))
    {
        return false;
    }
    name.get(tag.len())
        .is_none_or(|byte| !byte.is_ascii_alphanumeric())
}

fn authored_self_closing(source: &str, element: &ElementNode<'_>) -> bool {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    let Some(open) = source.as_bytes().get(start..end) else {
        return false;
    };
    let mut saw_close = false;
    for byte in open.iter().rev() {
        match *byte {
            b'>' if !saw_close => saw_close = true,
            b'/' if saw_close => return true,
            b' ' | b'\t' | b'\n' | b'\r' | 0x0c if saw_close => {}
            _ if saw_close => return false,
            _ => {}
        }
    }
    false
}

fn can_omit_end_tag(tag: &str) -> bool {
    OPTIONAL_END_TAGS
        .iter()
        .any(|name| tag.eq_ignore_ascii_case(name))
}

fn is_void_element(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    if bytes.len() > 24 {
        return false;
    }
    let mut lower = [0u8; 24];
    for (slot, byte) in lower.iter_mut().zip(bytes) {
        *slot = byte.to_ascii_lowercase();
    }
    lower
        .get(..bytes.len())
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .is_some_and(vize_l0::is_void_tag)
}

/// Offset after `</tag>` when it occupies `[from, limit)`, aside from
/// surrounding ASCII whitespace. Anything else means the end tag is absent.
fn closing_end(source: &str, from: u32, limit: u32, tag: &str) -> Option<u32> {
    let bytes = source.as_bytes();
    let mut at = from as usize;
    let limit = (limit as usize).min(bytes.len());
    if at > limit {
        return None;
    }
    while at < limit && bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    let tag = tag.as_bytes();
    let name_at = at.checked_add(2)?;
    let name_end = name_at.checked_add(tag.len())?;
    if name_end > limit || bytes.get(at..name_at) != Some(b"</") {
        return None;
    }
    if !bytes.get(name_at..name_end)?.eq_ignore_ascii_case(tag) {
        return None;
    }
    let mut close = name_end;
    while close < limit && bytes.get(close).is_some_and(u8::is_ascii_whitespace) {
        close += 1;
    }
    if bytes.get(close) == Some(&b'>') {
        u32::try_from(close + 1).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
