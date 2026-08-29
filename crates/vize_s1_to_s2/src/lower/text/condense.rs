//! The condense half of `lower::text`: the whitespace plan — the
//! armature algorithm (`crates/vize_armature/src/parser/whitespace.rs`)
//! expressed as per-index actions over the S1 child list, comments and
//! every other non-text child standing exactly where the legacy tree
//! holds them. The merge half (and the module-level decision record)
//! stays in `lower/text.rs`.

use alloc::vec::Vec as StdVec;

use vize_s0::{String, StringBuilder};
use vize_s1::SurfaceChild;

use super::super::cx::Cx;

/// Elements whose content is raw text, not template markup — exempt from
/// condensing (the metamorphic suite's list, `tests/metamorphic/sites.rs`).
pub(crate) const RAWTEXT_TAGS: [&str; 9] = [
    "script",
    "style",
    "textarea",
    "title",
    "iframe",
    "noscript",
    "xmp",
    "listing",
    "plaintext",
];

/// Whether `tag` suppresses condensing for its whole subtree: the
/// shipped `is_pre_tag` (`tag == "pre"`) plus the rawtext set.
pub(crate) fn suppresses_condense(tag: &str) -> bool {
    tag == "pre" || RAWTEXT_TAGS.contains(&tag)
}

/// Vue's whitespace alphabet for the condense strategy — exactly
/// `[ \t\n\f\r]` (`whitespace.rs:12-16`), never full-Unicode.
#[inline]
fn is_vue_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
}

/// The plan for one text child, computed list-wide before lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextAction<'a> {
    /// Lower as authored.
    Keep,
    /// Lower with this content (a condensed rewrite, or the single
    /// space of a condensed whitespace run).
    Content(&'a str),
    /// A removed whitespace-only text node.
    Drop,
}

/// A text-like child for the remove-vs-condense rule
/// (`whitespace.rs:181-187`): interpolations and non-whitespace text.
/// Comments, elements and every other kind are not.
fn text_like(child: &SurfaceChild<'_>) -> bool {
    match child {
        SurfaceChild::Interpolation(_) => true,
        SurfaceChild::Text(token) => !token.text.chars().all(is_vue_ws),
        _ => false,
    }
}

/// Collapse every maximal run of the alphabet in mixed text to one
/// space; `None` when the text already satisfies the strategy
/// (`whitespace.rs:22-61` — the untouched node keeps borrowing the
/// source).
fn condense_internal<'a>(cx: &Cx<'a>, text: &str) -> Option<&'a str> {
    let needs = {
        let mut prev_ws = false;
        let mut any = false;
        for c in text.chars() {
            if is_vue_ws(c) {
                if prev_ws || c != ' ' {
                    any = true;
                }
                prev_ws = true;
            } else {
                prev_ws = false;
            }
        }
        any
    };
    if !needs {
        return None;
    }
    let mut out = StringBuilder::with_capacity_in(text.len(), cx.allocator);
    let mut prev_ws = false;
    for c in text.chars() {
        if is_vue_ws(c) {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    Some(out.into_str())
}

/// The fused-part collapse: the same maximal-run rule as
/// [`condense_internal`], in place over an owned part — re-run after
/// static members fuse, so a whitespace run straddling a member seam
/// condenses exactly as the one-node spelling does. Idempotent over
/// already-collapsed content.
pub(super) fn collapse_fused(text: &mut String) {
    let needs = {
        let mut prev_ws = false;
        let mut any = false;
        for c in text.chars() {
            if is_vue_ws(c) {
                if prev_ws || c != ' ' {
                    any = true;
                }
                prev_ws = true;
            } else {
                prev_ws = false;
            }
        }
        any
    };
    if !needs {
        return;
    }
    let mut out = String::default();
    let mut prev_ws = false;
    for c in text.chars() {
        if is_vue_ws(c) {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    *text = out;
}

/// One contiguous text group: a maximal run of `Text` children whose
/// bytes tile without a gap (empty `leading`, each starting where the
/// previous ended). A parse emits maximal runs, so a group is one node
/// there; multi-member groups arise only from comment-free adjacency —
/// split trees and recovered shapes — where the members are **one DOM
/// text run** and must classify as one node — the split-mutator law
/// the metamorphic suite holds (its corpus canary caught the first cut
/// reading per node and stripping the whitespace-only half of a split
/// mixed run).
#[derive(Debug, Clone, Copy)]
struct TextGroup {
    start: usize,
    end: usize,
    ws_only: bool,
    has_newline: bool,
}

fn text_groups<'a>(cx: &Cx<'a>, children: &[SurfaceChild<'a>]) -> StdVec<TextGroup> {
    let mut groups = StdVec::new();
    let mut i = 0usize;
    while i < children.len() {
        let SurfaceChild::Text(token) = &children[i] else {
            i += 1;
            continue;
        };
        let start = i;
        let mut ws_only = token.text.chars().all(is_vue_ws);
        let mut has_newline = token.text.contains('\n') || token.text.contains('\r');
        let mut end_offset = cx.offset(token.text) + token.text.len() as u32;
        i += 1;
        while i < children.len() {
            let SurfaceChild::Text(next) = &children[i] else {
                break;
            };
            if !next.leading.is_empty() || cx.offset(next.text) != end_offset {
                // A byte gap (recovered junk) is a group boundary.
                break;
            }
            ws_only &= next.text.chars().all(is_vue_ws);
            has_newline |= next.text.contains('\n') || next.text.contains('\r');
            end_offset += next.text.len() as u32;
            i += 1;
        }
        groups.push(TextGroup {
            start,
            end: i,
            ws_only,
            has_newline,
        });
    }
    groups
}

/// The condense plan for one child list — the armature algorithm
/// (`whitespace.rs:69-165`) expressed as per-index actions on the S1
/// list, comments and every other non-text child standing exactly where
/// the legacy tree holds them. Classification runs over [`TextGroup`]s
/// (on parser output, one node each — the algorithm is then armature's
/// exactly). All-`Keep` inside a suppressed subtree.
pub(crate) fn plan_whitespace<'a>(
    cx: &Cx<'a>,
    children: &[SurfaceChild<'a>],
) -> StdVec<TextAction<'a>> {
    let mut plan: StdVec<TextAction<'a>> = StdVec::new();
    plan.resize(children.len(), TextAction::Keep);
    if cx.condense_suppressed() {
        return plan;
    }
    let groups = text_groups(cx, children);

    // Leading and trailing whitespace-only text is removed
    // unconditionally (`whitespace.rs:74-95`), group-wise.
    let mut first_group = 0usize;
    let mut last_group = groups.len();
    let mut lo = 0usize;
    while first_group < last_group {
        let group = &groups[first_group];
        if group.start != lo || !group.ws_only {
            break;
        }
        for slot in &mut plan[group.start..group.end] {
            *slot = TextAction::Drop;
        }
        lo = group.end;
        first_group += 1;
    }
    let mut hi = children.len();
    while last_group > first_group {
        let group = &groups[last_group - 1];
        if group.end != hi || !group.ws_only {
            break;
        }
        for slot in &mut plan[group.start..group.end] {
            *slot = TextAction::Drop;
        }
        hi = group.start;
        last_group -= 1;
    }

    for group in &groups[first_group..last_group] {
        if group.ws_only {
            // Group neighbours are the nearest non-text children (on
            // parser output exactly `whitespace.rs:107-113`).
            let prev_is_text = group.start > lo && text_like(&children[group.start - 1]);
            let next_is_text = group.end < hi && text_like(&children[group.end]);
            if !prev_is_text && !next_is_text && group.has_newline {
                for slot in &mut plan[group.start..group.end] {
                    *slot = TextAction::Drop;
                }
            } else {
                plan[group.start] = TextAction::Content(" ");
                for slot in &mut plan[group.start + 1..group.end] {
                    *slot = TextAction::Drop;
                }
            }
        } else {
            // A mixed group keeps every member; single-member interior
            // collapse happens here, and a multi-member group's collapse
            // runs over the fused content at merge time instead
            // (`collapse_fused` — the two compose to the same bytes).
            if group.end - group.start == 1
                && let SurfaceChild::Text(token) = &children[group.start]
                && !token.text.chars().all(is_vue_ws)
                && let Some(condensed) = condense_internal(cx, token.text)
            {
                plan[group.start] = TextAction::Content(condensed);
            }
        }
    }
    plan
}

/// Whether `child` may extend a merge run starting at `end`: a text or
/// interpolation child whose bytes begin exactly at `end` (no leading
/// gap — a comment, a dropped node or recovered junk between two
/// children is a hard run boundary, so a merged span is always the
/// authored bytes and the parts stay span-contiguous).
pub(super) fn extends_run(cx: &Cx<'_>, child: &SurfaceChild<'_>, end: u32) -> bool {
    match child {
        SurfaceChild::Text(token) => token.leading.is_empty() && cx.offset(token.text) == end,
        SurfaceChild::Interpolation(node) => {
            node.open.leading.is_empty() && cx.offset(node.open.text) == end
        }
        _ => false,
    }
}
