//! Guarded check blocks emitted as one `if` / `else if` chain per branch run.
//!
//! Each check under a `v-else-if` branch carries the whole branch guard: the
//! negated conditions of every earlier branch followed by its own. Emitted as
//! independent `if (guard) { … }` blocks, every check repeats the chain, so a
//! template with N branches costs O(N²) conditions and O(N³) narrowing work,
//! and a few dozen branches push one generated body past TypeScript's
//! control-flow depth limit. TypeScript then reports TS2563 on the generated
//! body and stops flow typing for every file it checks afterwards.
//!
//! The blocks of one emission pass are written back to back, so a block that
//! follows its predecessor directly is folded into it instead: the same guard
//! reopens the block, and the next branch of the same chain becomes
//! `else if (<own condition>)` inside one enclosing `if (<chain base>)`.
//! The enclosing guard must be factored out: the false branch of `P && A`
//! followed by `P` does not let TypeScript recover `!A`. Each condition still
//! appears once and sibling exclusions flow through the inner chain.
//!
//! A v-for whose body already sits in that `if` suspends the chain so inner
//! checks fold on their own, and records the active prefix. A guard that the
//! prefix fully covers emits no further `if`.

use std::cell::{Cell, RefCell};
use std::mem;

use vize_carton::{String, append};

use super::vif_guard::{append_ignored_vif_guard_open, split_guard_terms};

/// The block most recently closed by [`append_chained_vif_guard_close`].
struct ClosedGuard {
    /// Buffer length right after the closing brace.
    closed_at: usize,
    indent: String,
    /// Terms shared by every branch of the chain (the enclosing guard).
    base: Vec<String>,
    /// Every term of the guard the closed block was opened with.
    terms: Vec<String>,
    block: u64,
}

struct OpenGuard {
    base: Vec<String>,
    terms: Vec<String>,
    /// Identity of the emitted TypeScript block. A reopened block keeps it.
    block: u64,
}

enum Frame {
    Emitted,
    Skipped,
}

thread_local! {
    static OPEN: RefCell<Option<OpenGuard>> = const { RefCell::new(None) };
    static CLOSED: RefCell<Option<ClosedGuard>> = const { RefCell::new(None) };
    static NEXT_BLOCK: Cell<u64> = const { Cell::new(0) };
    static PREFIX: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) };
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
/// it when both belong to the same branch or to consecutive branches.
pub(crate) fn append_chained_vif_guard_open(
    ts: &mut String,
    indent: &str,
    guard: &str,
    purpose: &str,
) {
    open_with(ts, indent, guard, Some(purpose));
}

/// Same chain as [`append_chained_vif_guard_open`] without an `@ts-ignore` line.
/// Prop replay guards are unmapped, so the comment would only add text.
pub(super) fn append_bare_chained_vif_guard_open(ts: &mut String, indent: &str, guard: &str) {
    open_with(ts, indent, guard, None);
}

fn open_with(ts: &mut String, indent: &str, guard: &str, purpose: Option<&str>) {
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let mut terms: Vec<String> = split_guard_terms(isolated.as_str())
        .into_iter()
        .map(String::from)
        .collect();
    let stripped = strip_active_prefix(&mut terms);
    if stripped && terms.is_empty() {
        FRAMES.with(|frames| frames.borrow_mut().push(Frame::Skipped));
        return;
    }
    FRAMES.with(|frames| frames.borrow_mut().push(Frame::Emitted));
    let joined = stripped.then(|| terms.join(" && "));
    let guard = joined.as_deref().unwrap_or(guard);
    let previous = CLOSED.with(|closed| closed.borrow_mut().take());
    let mut block = None;
    let base = match previous {
        Some(previous) if follows_directly(&previous, ts, indent) => {
            match continuation(&previous.terms, &terms) {
                Some(Continuation::SameBranch) => {
                    reopen(
                        ts,
                        indent,
                        previous.closed_at,
                        closing_blocks(&previous.base),
                    );
                    block = Some(previous.block);
                    previous.base
                }
                Some(Continuation::NextBranch(own)) => {
                    if !previous.base.is_empty() {
                        // Keep the previous branch closed, but reopen its enclosing guard.
                        reopen(ts, indent, previous.closed_at, 1);
                    }
                    if let Some(purpose) = purpose {
                        append!(
                            *ts,
                            "{indent}// @ts-ignore {purpose}; authored v-if checks own diagnostics.\n"
                        );
                    }
                    if own.is_empty() {
                        append!(*ts, "{indent}else {{\n");
                    } else {
                        let condition = own.join(" && ");
                        append!(*ts, "{indent}else if ({condition}) {{\n");
                    }
                    previous.base
                }
                None => open_fresh(ts, indent, guard, purpose, &terms),
            }
        }
        _ => open_fresh(ts, indent, guard, purpose, &terms),
    };
    let block = block.unwrap_or_else(next_block);
    OPEN.with(|open| *open.borrow_mut() = Some(OpenGuard { base, terms, block }));
}

/// Close a block opened by [`append_chained_vif_guard_open`].
pub(crate) fn append_chained_vif_guard_close(ts: &mut String, indent: &str) {
    let skipped = FRAMES.with(|frames| matches!(frames.borrow_mut().pop(), Some(Frame::Skipped)));
    if skipped {
        return;
    }
    append!(*ts, "{indent}}}\n");
    let Some(open) = OPEN.with(|open| open.borrow_mut().take()) else {
        return;
    };
    if !open.base.is_empty() {
        append!(*ts, "{indent}}}\n");
    }
    CLOSED.with(|closed| {
        *closed.borrow_mut() = Some(ClosedGuard {
            closed_at: ts.len(),
            indent: String::from(indent),
            base: open.base,
            terms: open.terms,
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

fn open_fresh(
    ts: &mut String,
    indent: &str,
    guard: &str,
    purpose: Option<&str>,
    terms: &[String],
) -> Vec<String> {
    let Some((last, base)) = terms.split_last() else {
        emit_if(ts, indent, guard, purpose);
        return Vec::new();
    };
    if base.is_empty() {
        emit_if(ts, indent, guard, purpose);
    } else {
        emit_if(ts, indent, &base.join(" && "), purpose);
        emit_if(ts, indent, last, purpose);
    }
    base.to_vec()
}

fn emit_if(ts: &mut String, indent: &str, guard: &str, purpose: Option<&str>) {
    if let Some(purpose) = purpose {
        append_ignored_vif_guard_open(ts, indent, guard, purpose);
        return;
    }
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    append!(*ts, "{indent}if ({}) {{\n", isolated.as_str());
}

fn closing_blocks(base: &[String]) -> usize {
    if base.is_empty() { 1 } else { 2 }
}

/// Whether `ts` still ends with the closing brace of `previous`, followed by
/// nothing but comment lines. Generation of one SFC runs on one thread and
/// writes one buffer at a time, so the thread-local record cannot describe a
/// different buffer that happens to end the same way.
fn follows_directly(previous: &ClosedGuard, ts: &String, indent: &str) -> bool {
    let close_len = (indent.len() + 2) * closing_blocks(&previous.base);
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

/// Drop the closing brace written by the previous close, keeping any comment
/// lines written after it so they move into the reopened block.
fn reopen(ts: &mut String, indent: &str, closed_at: usize, blocks: usize) {
    let between = String::from(ts.get(closed_at..).unwrap_or_default());
    let close_len = (indent.len() + 2) * blocks;
    ts.truncate(closed_at.saturating_sub(close_len));
    ts.push_str(between.as_str());
}

enum Continuation {
    SameBranch,
    /// The new block is the next branch; carries its own terms.
    NextBranch(Vec<String>),
}

/// `previous = P + [t]`, `next = P + [!t] + own` with `own` empty for `v-else`.
fn continuation(previous: &[String], next: &[String]) -> Option<Continuation> {
    if previous == next {
        return Some(Continuation::SameBranch);
    }
    let (last, prefix) = previous.split_last()?;
    if !last.starts_with('(') || next.len() < previous.len() {
        return None;
    }
    let (shared, rest) = next.split_at_checked(prefix.len())?;
    let (negated, own) = rest.split_first()?;
    let expected = {
        let mut negated_last = String::from("!");
        negated_last.push_str(last.as_str());
        negated_last
    };
    (shared == prefix && *negated == expected && own.len() <= 1)
        .then(|| Continuation::NextBranch(own.to_vec()))
}

#[cfg(test)]
mod chain_tests;
