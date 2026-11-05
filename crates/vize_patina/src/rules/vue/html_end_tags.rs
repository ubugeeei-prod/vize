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

#[derive(Default)]
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
    if let Some(end) = closing_end(ctx.source, inner.end, limit, element.tag) {
        return Extent {
            end,
            reliable: true,
        };
    }
    if can_omit_end_tag(element.tag) {
        report_missing(ctx, element, inner.end);
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
    if name.len() < tag.len() || !name[..tag.len()].eq_ignore_ascii_case(tag) {
        return false;
    }
    name.len() == tag.len() || !name[tag.len()].is_ascii_alphanumeric()
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
    std::str::from_utf8(&lower[..bytes.len()]).is_ok_and(|lowered| vize_l0::is_void_tag(lowered))
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
    while at < limit && bytes[at].is_ascii_whitespace() {
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
    while close < limit && bytes[close].is_ascii_whitespace() {
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
