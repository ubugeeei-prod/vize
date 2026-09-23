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

/// Whether `tag` suppresses condensing for its whole subtree: the
/// shipped `is_pre_tag` (`tag == "pre"`).
pub(crate) fn suppresses_condense(tag: &str) -> bool {
    tag == "pre"
}

/// Vue's whitespace alphabet for the condense strategy — exactly
/// `[ \t\n\f\r]` (`whitespace.rs:12-16`), never full-Unicode.
#[inline]
pub(super) fn is_vue_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
}

/// Whether `child` is a comment this compile is not preserving — a
/// child the shipped lane never built, so the text rules must look past
/// it in every direction.
fn invisible_comment(cx: &Cx<'_>, child: &SurfaceChild<'_>) -> bool {
    !cx.preserve_comments() && matches!(child, SurfaceChild::Comment(_))
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
    /// How many **text** children the group holds. Not `end - start`:
    /// an absorbed dropped comment occupies an index and no text.
    texts: usize,
    /// Index of the group's first text child. Not `start`: a dropped
    /// comment can open the run (`<!--c-->\n  a`).
    first_text: usize,
    ws_only: bool,
    has_newline: bool,
}

fn text_groups<'a>(cx: &Cx<'a>, children: &[SurfaceChild<'a>]) -> StdVec<TextGroup> {
    let mut groups = StdVec::new();
    let mut i = 0usize;
    while i < children.len() {
        // A group is a byte-contiguous run of text children and of the
        // comments this compile is not preserving, holding at least one
        // text child. Absorbing the comments is what makes the rules
        // below see the child list the shipped lane sees: Vue's parser
        // builds no node for a dropped comment, so the text either side
        // of one is a single node to it and this run is a single group
        // to us. A byte gap (recovered junk) still ends the run, so the
        // group covers exactly the authored bytes.
        let start = i;
        let mut ws_only = true;
        let mut has_newline = false;
        let mut texts = 0usize;
        let mut first_text = start;
        let mut end_offset = None;
        while let Some(child) = children.get(i) {
            let (offset, len, end) = match child {
                SurfaceChild::Text(token) if token.leading.is_empty() => {
                    (cx.offset(token.text), token.text.len() as u32, None::<u32>)
                }
                SurfaceChild::Comment(token)
                    if invisible_comment(cx, child) && token.leading.is_empty() =>
                {
                    (cx.offset(token.text), 0, Some(cx.token_span(token).end))
                }
                _ => break,
            };
            if end_offset.is_some_and(|at| at != offset) {
                break;
            }
            if let SurfaceChild::Text(token) = child {
                if texts == 0 {
                    first_text = i;
                }
                ws_only &= token.text.chars().all(is_vue_ws);
                has_newline |= token.text.contains('\n') || token.text.contains('\r');
                texts += 1;
                end_offset = Some(offset + len);
            } else {
                end_offset = end;
            }
            i += 1;
        }
        if texts == 0 {
            // A comment-only run is nobody's group. Every consumed
            // member was an invisible comment, so skip the whole run
            // instead of rescanning each suffix. A non-groupable first
            // child consumed nothing and still needs one step forward.
            i = i.max(start + 1);
            continue;
        }
        groups.push(TextGroup {
            start,
            end: i,
            texts,
            first_text,
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
    for group in &groups {
        if group.start != lo || !group.ws_only {
            break;
        }
        drop_members(&mut plan, group);
        lo = group.end;
        first_group += 1;
    }
    let mut hi = children.len();
    for group in groups.get(first_group..).unwrap_or_default().iter().rev() {
        if group.end != hi || !group.ws_only {
            break;
        }
        drop_members(&mut plan, group);
        hi = group.start;
        last_group -= 1;
    }

    for group in groups.get(first_group..last_group).unwrap_or_default() {
        if group.ws_only {
            // Group neighbours are the nearest non-text children (on
            // parser output exactly `whitespace.rs:107-113`). This is
            // the whole rule in both comment configurations: a comment
            // the compile preserves is a real, non-text-like neighbour,
            // and one it drops was already absorbed into this group by
            // `text_groups`, so it is not a neighbour at all. The four
            // comment-edge special cases this arm used to carry were
            // emulating the second case from inside the first, and
            // measured against `@vue/compiler-dom` they got it wrong on
            // the shapes `dropped_comment_text_runs` now pins.
            let prev_is_text = group.start > lo
                && group
                    .start
                    .checked_sub(1)
                    .and_then(|before| children.get(before))
                    .is_some_and(text_like);
            let next_is_text = group.end < hi && children.get(group.end).is_some_and(text_like);
            if !prev_is_text && !next_is_text && group.has_newline {
                drop_members(&mut plan, group);
            } else {
                // The condensed space belongs on the group's first
                // **text** child: an absorbed dropped comment can open
                // the group (`<i/><!--c--> <b/>`), and writing the
                // content to its index would drop the space Vue keeps.
                for (index, slot) in members(&mut plan, group).iter_mut().enumerate() {
                    *slot = if group.start + index == group.first_text {
                        TextAction::Content(" ")
                    } else {
                        TextAction::Drop
                    };
                }
            }
        } else {
            // A mixed group keeps every member; single-member interior
            // collapse happens here, and a multi-member group's collapse
            // runs over the fused content at merge time instead
            // (`collapse_fused` — the two compose to the same bytes).
            if group.texts == 1
                && let Some(SurfaceChild::Text(token)) = children.get(group.first_text)
                && !token.text.chars().all(is_vue_ws)
                && let Some(condensed) = condense_internal(cx, token.text)
                && let Some(slot) = plan.get_mut(group.first_text)
            {
                *slot = TextAction::Content(condensed);
            }
        }
    }
    plan
}

/// A group's members in the plan (the plan holds one slot per child).
fn members<'p, 'a>(plan: &'p mut [TextAction<'a>], group: &TextGroup) -> &'p mut [TextAction<'a>] {
    plan.get_mut(group.start..group.end).unwrap_or_default()
}

fn drop_members(plan: &mut [TextAction<'_>], group: &TextGroup) {
    for slot in members(plan, group) {
        *slot = TextAction::Drop;
    }
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
