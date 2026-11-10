//! Guarded check blocks folded into one nested `if` tree per run.
//!
//! Each check under a `v-else-if` branch carries the whole branch guard: the
//! negated conditions of every earlier branch followed by its own, plus the
//! guards of every enclosing `v-if`. Emitted as independent `if (guard) { … }`
//! blocks, every check repeats the chain, so a template with N branches costs
//! O(N²) conditions and O(N³) narrowing work, and a few dozen branches push
//! one generated body past TypeScript's control-flow depth limit. TypeScript
//! then reports TS2563 on the generated body and stops flow typing for every
//! file it checks afterwards. Independent blocks are also costly for callbacks
//! created after them: each callback is its own flow container, so its
//! narrowing walk crosses every earlier block body again.
//!
//! A guard is emitted as levels: a run of exclusions, then each positive term.
//! A block that follows its predecessor directly (only comments in between)
//! reuses the predecessor's levels that its guard starts with, continues the
//! first differing level with `else` / `else if (<own condition>)` when the
//! guard is that level's next branch, and nests whatever terms remain. The
//! false branch of `P && A` followed by `P` does not let TypeScript recover
//! `!A`, so `else` only continues a level whose `if` tests one positive term.
//!
//! A v-for whose body already sits in that `if` suspends the chain so inner
//! checks fold on their own, and records the active prefix. A guard that the
//! prefix fully covers emits no further `if`.

use std::cell::{Cell, RefCell};
use std::mem;
use std::ops::Range;

use vize_carton::{String, append};

use super::vif_guard::{append_ignored_vif_guard_open, split_guard_terms};

/// One `if` level of an emitted guard block.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Frame {
    /// Terms this level contributes to the block's guard, in order. An
    /// `else` level also carries the exclusions its position implies.
    terms: Vec<String>,
    /// The condition when the level's `if` tests exactly one positive term,
    /// so a following branch can continue it with `else`.
    branch: Option<String>,
}

/// The block most recently closed by [`append_chained_vif_guard_close`].
struct ClosedGuard {
    /// Buffer length right after the last closing brace.
    closed_at: usize,
    indent: String,
    frames: Vec<Frame>,
    block: u64,
}

struct OpenGuard {
    frames: Vec<Frame>,
    /// Identity of the emitted TypeScript block. A reopened block keeps it.
    block: u64,
}

enum FrameState {
    Emitted,
    Skipped,
}

thread_local! {
    static OPEN: RefCell<Option<OpenGuard>> = const { RefCell::new(None) };
    static CLOSED: RefCell<Option<ClosedGuard>> = const { RefCell::new(None) };
    static NEXT_BLOCK: Cell<u64> = const { Cell::new(0) };
    static PREFIX: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static FRAMES: RefCell<Vec<FrameState>> = const { RefCell::new(Vec::new()) };
}

/// The open chain around a v-for body. Inner checks must not replace it.
pub(super) struct SuspendedChain {
    open: Option<OpenGuard>,
    closed: Option<ClosedGuard>,
}

impl SuspendedChain {
    pub(super) fn suspend() -> Self {
        Self {
            open: OPEN.with(|open| open.borrow_mut().take()),
            closed: CLOSED.with(|closed| closed.borrow_mut().take()),
        }
    }
}

impl Drop for SuspendedChain {
    fn drop(&mut self) {
        OPEN.with(|open| *open.borrow_mut() = self.open.take());
        CLOSED.with(|closed| *closed.borrow_mut() = self.closed.take());
    }
}

/// Guard terms already enforced by a surrounding `if`.
pub(super) struct ActiveGuardPrefix {
    previous: Vec<String>,
}

impl ActiveGuardPrefix {
    pub(super) fn push(guard: &str) -> Self {
        let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
        let terms = split_guard_terms(isolated.as_str())
            .into_iter()
            .map(String::from)
            .collect();
        let previous = PREFIX.with(|prefix| mem::replace(&mut *prefix.borrow_mut(), terms));
        Self { previous }
    }
}

impl Drop for ActiveGuardPrefix {
    fn drop(&mut self) {
        PREFIX.with(|prefix| *prefix.borrow_mut() = mem::take(&mut self.previous));
    }
}

/// The emitted block a check written right now lands in, if it is guarded.
/// Declarations made in one block are not visible from another block with the
/// same guard, so anything shared between checks must be keyed by it.
pub(crate) fn open_chained_vif_guard_block() -> Option<u64> {
    OPEN.with(|open| open.borrow().as_ref().map(|open| open.block))
}

fn next_block() -> u64 {
    NEXT_BLOCK.with(|next| {
        let block = next.get();
        next.set(block + 1);
        block
    })
}

/// Open a guarded check block, folding it into the block closed right before
/// it when their guards share levels.
pub(crate) fn append_chained_vif_guard_open(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
    purpose: &str,
) {
    open_with(ts, indent, guard, Some(purpose));
}

/// Same chain as [`append_chained_vif_guard_open`] without an `@ts-ignore` line.
/// Prop replay guards are unmapped, so the comment would only add text.
pub(super) fn append_bare_chained_vif_guard_open(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
) {
    open_with(ts, indent, guard, None);
}

/// Open a chained block whose innermost level tests the authored `v-if`
/// condition (the guard's last term) without `@ts-ignore`, so diagnostics in
/// the condition stay reportable. Returns the byte range of that `if` /
/// `else if` line, or `None` when the guard cannot be split; the caller then
/// writes its own block.
pub(crate) fn append_chained_vif_condition_open(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
    purpose: &str,
) -> Option<Range<usize>> {
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    if isolated.is_owned() {
        return None;
    }
    let terms: Vec<String> = split_guard_terms(isolated.as_str())
        .into_iter()
        .map(String::from)
        .collect();
    let (_, prefix) = terms.split_last()?;
    let mut prefix = prefix.to_vec();
    let stripped = strip_active_prefix(&mut prefix);
    let terms: Vec<String> = if stripped {
        prefix.iter().chain(terms.last()).cloned().collect()
    } else {
        terms
    };
    FRAMES.with(|frames| frames.borrow_mut().push(FrameState::Emitted));
    let previous = CLOSED
        .with(|closed| closed.borrow_mut().take())
        .filter(|previous| follows_directly(previous, ts, indent));
    let mut condition = None;
    let (frames, _) = match previous {
        Some(previous) => fold(
            ts,
            indent,
            Some(purpose),
            &terms,
            previous,
            Some(&mut condition),
        ),
        None => (
            open_levels(ts, indent, Some(purpose), &terms, Some(&mut condition)),
            0,
        ),
    };
    let block = next_block();
    OPEN.with(|open| *open.borrow_mut() = Some(OpenGuard { frames, block }));
    condition
}

fn open_with(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
    purpose: Option<&str>,
) {
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let mut terms: Vec<String> = split_guard_terms(isolated.as_str())
        .into_iter()
        .map(String::from)
        .collect();
    let stripped = strip_active_prefix(&mut terms);
    if stripped && terms.is_empty() {
        FRAMES.with(|frames| frames.borrow_mut().push(FrameState::Skipped));
        return;
    }
    FRAMES.with(|frames| frames.borrow_mut().push(FrameState::Emitted));
    if terms.is_empty() {
        // Nothing to split or share: keep the authored text as one level.
        emit_if(ts, indent, isolated.as_str(), purpose);
        let frames = vec![Frame {
            terms: Vec::new(),
            branch: None,
        }];
        let block = next_block();
        OPEN.with(|open| *open.borrow_mut() = Some(OpenGuard { frames, block }));
        return;
    }
    let previous = CLOSED
        .with(|closed| closed.borrow_mut().take())
        .filter(|previous| follows_directly(previous, ts, indent));
    let (frames, block) = match previous {
        Some(previous) => fold(ts, indent, purpose, &terms, previous, None),
        None => (open_levels(ts, indent, purpose, &terms, None), next_block()),
    };
    OPEN.with(|open| *open.borrow_mut() = Some(OpenGuard { frames, block }));
}

/// Fold `terms` into the levels of the block that was just closed.
fn fold(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    purpose: Option<&str>,
    terms: &[String],
    previous: ClosedGuard,
    mut condition: Option<&mut Option<Range<usize>>>,
) -> (Vec<Frame>, u64) {
    // A mapped condition is always written again: its own level is never reused.
    let reusable = terms.len().saturating_sub(usize::from(condition.is_some()));
    let mut shared = 0;
    let mut covered = 0;
    for frame in &previous.frames {
        let end = covered + frame.terms.len();
        if frame.terms.is_empty()
            || end > reusable
            || terms.get(covered..end) != Some(frame.terms.as_slice())
        {
            break;
        }
        shared += 1;
        covered = end;
    }
    let total = previous.frames.len();
    if condition.is_none() && shared == total && covered == terms.len() {
        reopen(ts, indent, previous.closed_at, total);
        return (previous.frames, previous.block);
    }
    let mut frames: Vec<Frame> = previous.frames.get(..shared).unwrap_or_default().to_vec();
    let rest = terms.get(covered..).unwrap_or_default();
    if let Some(sibling) = previous
        .frames
        .get(shared)
        .and_then(|frame| sibling(frame, rest))
    {
        // Keep the earlier branch (and anything nested in it) closed and
        // continue its `if` from the enclosing levels.
        reopen(ts, indent, previous.closed_at, shared);
        let consumed = sibling.level.len();
        let mapped_here = covered + consumed == terms.len() && sibling.own.is_some();
        let mapped_condition = condition.as_deref_mut().filter(|_| mapped_here);
        // Authored v-if checks own this sibling guard's diagnostics.
        if let (Some(_), None) = (purpose, mapped_condition.as_ref()) {
            append!(*ts, "{indent}// @ts-ignore\n");
        }
        let start = ts.len();
        match sibling.own.as_deref() {
            Some(own) => append!(*ts, "{indent}else if ({own}) {{\n"),
            None => append!(*ts, "{indent}else {{\n"),
        }
        if let Some(mapped_condition) = mapped_condition {
            *mapped_condition = Some(start..ts.len());
        }
        let condition = condition.filter(|_| !mapped_here);
        frames.push(Frame {
            terms: sibling.level,
            branch: sibling.own,
        });
        frames.extend(open_levels(
            ts,
            indent,
            purpose,
            rest.get(consumed..).unwrap_or_default(),
            condition,
        ));
        return (frames, next_block());
    }
    reopen(ts, indent, previous.closed_at, shared);
    frames.extend(open_levels(ts, indent, purpose, rest, condition));
    (frames, next_block())
}

struct Sibling {
    /// Terms of the continued level: the earlier branch's exclusions, the
    /// negated earlier condition and the own condition.
    level: Vec<String>,
    own: Option<String>,
}

/// `frame = E + [t]`, `rest = E + [!t] + [own]? + nested` continues the same
/// `if` with `else if (own)` or `else`.
fn sibling(frame: &Frame, rest: &[String]) -> Option<Sibling> {
    let condition = frame.branch.as_ref()?;
    let exclusions = frame.terms.split_last()?.1;
    if rest.get(..exclusions.len())? != exclusions {
        return None;
    }
    let negated = rest.get(exclusions.len())?;
    if negated.strip_prefix('!') != Some(condition.as_str()) {
        return None;
    }
    let mut level = rest.get(..=exclusions.len())?.to_vec();
    let own = rest
        .get(exclusions.len() + 1)
        .filter(|term| !term.starts_with('!'))
        .cloned();
    if let Some(own) = &own {
        level.push(own.clone());
    }
    Some(Sibling { level, own })
}

/// Emit `terms` as fresh levels: each run of exclusions, then each positive
/// term on its own so later branches can continue it with `else`.
fn open_levels(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    purpose: Option<&str>,
    terms: &[String],
    condition: Option<&mut Option<Range<usize>>>,
) -> Vec<Frame> {
    let mapped = condition.is_some();
    let (body, last) = match terms.split_last() {
        Some((last, body)) if mapped => (body, Some(last)),
        _ => (terms, None),
    };
    let mut frames = Vec::new();
    let mut exclusions: Vec<String> = Vec::new();
    for term in body {
        if term.starts_with('!') {
            exclusions.push(term.clone());
            continue;
        }
        if !exclusions.is_empty() {
            emit_if(ts, indent, &exclusions.join(" && "), purpose);
            frames.push(Frame {
                terms: mem::take(&mut exclusions),
                branch: None,
            });
        }
        emit_if(ts, indent, term, purpose);
        frames.push(Frame {
            terms: vec![term.clone()],
            branch: Some(term.clone()),
        });
    }
    if let (Some(last), Some(condition)) = (last, condition) {
        if !exclusions.is_empty() {
            emit_if(ts, indent, &exclusions.join(" && "), purpose);
            frames.push(Frame {
                terms: mem::take(&mut exclusions),
                branch: None,
            });
        }
        let start = ts.len();
        append!(*ts, "{indent}if ({last}) {{\n");
        *condition = Some(start..ts.len());
        frames.push(Frame {
            terms: vec![last.clone()],
            branch: (!last.starts_with('!')).then(|| last.clone()),
        });
        return frames;
    }
    if !exclusions.is_empty() {
        emit_if(ts, indent, &exclusions.join(" && "), purpose);
        frames.push(Frame {
            terms: exclusions,
            branch: None,
        });
    }
    frames
}

/// Close a block opened by [`append_chained_vif_guard_open`].
pub(crate) fn append_chained_vif_guard_close(ts: &mut crate::virtual_ts::TextBuf, indent: &str) {
    let skipped =
        FRAMES.with(|frames| matches!(frames.borrow_mut().pop(), Some(FrameState::Skipped)));
    if skipped {
        return;
    }
    let Some(open) = OPEN.with(|open| open.borrow_mut().take()) else {
        append!(*ts, "{indent}}}\n");
        return;
    };
    for _ in &open.frames {
        append!(*ts, "{indent}}}\n");
    }
    CLOSED.with(|closed| {
        *closed.borrow_mut() = Some(ClosedGuard {
            closed_at: ts.len(),
            indent: String::from(indent),
            frames: open.frames,
            block: open.block,
        });
    });
}

/// Remove a leading prefix the surrounding `if` already enforces.
/// An empty prefix does no extra work. A prefix that is not an exact leading
/// term sequence leaves the guard unchanged.
fn strip_active_prefix(terms: &mut Vec<String>) -> bool {
    PREFIX.with(|prefix| {
        let prefix = prefix.borrow();
        if prefix.is_empty()
            || prefix.len() > terms.len()
            || !terms
                .iter()
                .zip(prefix.iter())
                .all(|(term, expected)| term == expected)
        {
            return false;
        }
        terms.drain(..prefix.len());
        true
    })
}

fn emit_if(ts: &mut crate::virtual_ts::TextBuf, indent: &str, guard: &str, purpose: Option<&str>) {
    if let Some(purpose) = purpose {
        append_ignored_vif_guard_open(ts, indent, guard, purpose);
        return;
    }
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    append!(*ts, "{indent}if ({}) {{\n", isolated.as_str());
}

/// Whether `ts` still ends with the closing braces of `previous`, followed by
/// nothing but comment lines. Generation of one SFC runs on one thread and
/// writes one buffer at a time, so the thread-local record cannot describe a
/// different buffer that happens to end the same way.
fn follows_directly(previous: &ClosedGuard, ts: &str, indent: &str) -> bool {
    let close_len = (indent.len() + 2) * previous.frames.len();
    let closing = previous
        .closed_at
        .checked_sub(close_len)
        .and_then(|start| ts.get(start..previous.closed_at));
    previous.indent == indent
        && closing.is_some_and(|closing| {
            closing
                .lines()
                .all(|line| line.strip_prefix(indent) == Some("}"))
        })
        && ts.get(previous.closed_at..).is_some_and(|between| {
            between.lines().all(|line| {
                let line = line.trim();
                line.is_empty() || line.starts_with("//")
            })
        })
}

/// Drop the last `blocks` closing braces written by the previous close,
/// keeping any comment lines written after them so they move into the
/// reopened block.
fn reopen(ts: &mut crate::virtual_ts::TextBuf, indent: &str, closed_at: usize, blocks: usize) {
    if blocks == 0 {
        return;
    }
    let between = String::from(ts.get(closed_at..).unwrap_or_default());
    let close_len = (indent.len() + 2) * blocks;
    ts.truncate(closed_at.saturating_sub(close_len));
    ts.push_str(between.as_str());
}

#[cfg(test)]
mod chain_tests;
