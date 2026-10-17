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
//! `else if (<own condition>)` inside one enclosing guard. Repeating that
//! enclosing guard in every branch loses TypeScript's discrimination: the
//! failure of `if (parent && child)` does not by itself imply `!child`.

use std::cell::RefCell;

use vize_carton::{String, append};

use super::vif_guard::{append_ignored_vif_guard_open, split_guard_terms};

/// The block most recently closed by [`append_chained_vif_guard_close`].
struct ClosedGuard {
    /// Buffer length right after the closing brace.
    closed_at: usize,
    /// Closing bytes for the branch and, when present, its enclosing guard.
    close_len: usize,
    indent: String,
    /// Terms shared by every branch of the chain (the enclosing guard).
    base: Vec<String>,
    /// Every term of the guard the closed block was opened with.
    terms: Vec<String>,
}

struct OpenGuard {
    base: Vec<String>,
    terms: Vec<String>,
}

thread_local! {
    static OPEN: RefCell<Option<OpenGuard>> = const { RefCell::new(None) };
    static CLOSED: RefCell<Option<ClosedGuard>> = const { RefCell::new(None) };
}

/// Open a guarded check block, folding it into the block closed right before
/// it when both belong to the same branch or to consecutive branches.
pub(crate) fn append_chained_vif_guard_open(
    ts: &mut String,
    indent: &str,
    guard: &str,
    purpose: &str,
) {
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let terms: Vec<String> = split_guard_terms(isolated.as_str())
        .into_iter()
        .map(String::from)
        .collect();
    let previous = CLOSED.with(|closed| closed.borrow_mut().take());
    let base = match previous {
        Some(previous) if follows_directly(&previous, ts, indent) => {
            match continuation(&previous.terms, &terms) {
                Some(Continuation::SameBranch) => {
                    reopen(ts, previous.closed_at, previous.close_len);
                    previous.base
                }
                Some(Continuation::NextBranch(own)) => {
                    if !previous.base.is_empty() {
                        // Keep the inner branch closed, but reopen its parent
                        // so this `else` inherits the preceding exclusions.
                        reopen(ts, previous.closed_at, indent.len() + 2);
                    }
                    append!(
                        *ts,
                        "{indent}// @ts-ignore {purpose}; authored v-if checks own diagnostics.\n"
                    );
                    if own.is_empty() {
                        append!(*ts, "{indent}else {{\n");
                    } else {
                        let condition = own
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .join(" && ");
                        append!(*ts, "{indent}else if ({condition}) {{\n");
                    }
                    previous.base
                }
                None => open_fresh(ts, indent, guard, purpose, &terms),
            }
        }
        _ => open_fresh(ts, indent, guard, purpose, &terms),
    };
    OPEN.with(|open| *open.borrow_mut() = Some(OpenGuard { base, terms }));
}

/// Close a block opened by [`append_chained_vif_guard_open`].
pub(crate) fn append_chained_vif_guard_close(ts: &mut String, indent: &str) {
    append!(*ts, "{indent}}}\n");
    let Some(open) = OPEN.with(|open| open.borrow_mut().take()) else {
        return;
    };
    let close_len = if open.base.is_empty() {
        indent.len() + 2
    } else {
        append!(*ts, "{indent}}}\n");
        2 * (indent.len() + 2)
    };
    CLOSED.with(|closed| {
        *closed.borrow_mut() = Some(ClosedGuard {
            closed_at: ts.len(),
            close_len,
            indent: String::from(indent),
            base: open.base,
            terms: open.terms,
        });
    });
}

fn open_fresh(
    ts: &mut String,
    indent: &str,
    guard: &str,
    purpose: &str,
    terms: &[String],
) -> Vec<String> {
    let Some((own, base)) = terms.split_last() else {
        append_ignored_vif_guard_open(ts, indent, guard, purpose);
        return Vec::new();
    };
    if base.is_empty() {
        append_ignored_vif_guard_open(ts, indent, guard, purpose);
    } else {
        let enclosing = base
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" && ");
        append_ignored_vif_guard_open(ts, indent, &enclosing, purpose);
        append_ignored_vif_guard_open(ts, indent, own.as_str(), purpose);
    }
    base.to_vec()
}

/// Whether `ts` still ends with the closing brace of `previous`, followed by
/// nothing but comment lines. Generation of one SFC runs on one thread and
/// writes one buffer at a time, so the thread-local record cannot describe a
/// different buffer that happens to end the same way.
fn follows_directly(previous: &ClosedGuard, ts: &String, indent: &str) -> bool {
    let closing = previous
        .closed_at
        .checked_sub(previous.close_len)
        .and_then(|start| ts.get(start..previous.closed_at));
    previous.indent == indent
        && closing.is_some_and(|closing| {
            closing
                .lines()
                .all(|line| line.trim() == "}" && line.starts_with(indent))
        })
        && ts.get(previous.closed_at..).is_some_and(|between| {
            between.lines().all(|line| {
                let line = line.trim();
                line.is_empty() || line.starts_with("//")
            })
        })
}

/// Drop closing braces written by the previous close, keeping any comment
/// lines written after it so they move into the reopened block.
fn reopen(ts: &mut String, closed_at: usize, close_len: usize) {
    let between = String::from(ts.get(closed_at..).unwrap_or_default());
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
mod tests {
    use super::{append_chained_vif_guard_close, append_chained_vif_guard_open};
    use vize_carton::{String, append};

    fn emit(guards: &[&str]) -> String {
        let mut ts = String::from("");
        for (index, guard) in guards.iter().enumerate() {
            append_chained_vif_guard_open(&mut ts, "  ", guard, "Inference-only guard");
            append!(ts, "    void check{index};\n");
            append_chained_vif_guard_close(&mut ts, "  ");
        }
        ts
    }

    #[test]
    fn consecutive_branches_become_one_else_if_chain() {
        let ts = emit(&[
            "(a)",
            "(a)",
            "!(a) && (b)",
            "!(a) && !(b) && (c)",
            "!(a) && !(b) && !(c)",
        ]);
        let conditions: Vec<&str> = ts
            .lines()
            .filter(|line| line.contains("if (") || line.trim() == "else {")
            .map(str::trim)
            .collect();
        assert_eq!(
            conditions,
            ["if ((a)) {", "else if ((b)) {", "else if ((c)) {", "else {"]
        );
        assert_eq!(ts.matches("void check").count(), 5);
        assert_eq!(ts.matches('{').count(), ts.matches('}').count());
    }

    #[test]
    fn an_enclosing_guard_wraps_the_entire_chain() {
        let ts = emit(&["(p) && (a)", "(p) && !(a) && (b)", "(p) && !(a) && !(b)"]);
        let conditions: Vec<&str> = ts
            .lines()
            .filter(|line| line.contains("if (") || line.trim() == "else {")
            .map(str::trim)
            .collect();
        assert_eq!(
            conditions,
            ["if ((p)) {", "if ((a)) {", "else if ((b)) {", "else {"],
            "{ts}"
        );
        assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
    }

    #[test]
    fn repeated_nested_checks_reopen_both_closing_braces() {
        let ts = emit(&[
            "(p) && (a)",
            "(p) && (a)",
            "(p) && !(a) && (b)",
            "(p) && !(a) && (b)",
            "(p) && !(a) && !(b)",
        ]);
        assert_eq!(ts.matches("if ((p))").count(), 1, "{ts}");
        assert_eq!(ts.matches("else if ((b))").count(), 1, "{ts}");
        assert_eq!(ts.matches("else {").count(), 1, "{ts}");
        assert_eq!(ts.matches("void check").count(), 5, "{ts}");
        assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
    }

    #[test]
    fn a_long_nested_chain_emits_each_condition_once() {
        let mut guard = String::from("(parent)");
        let mut guards = Vec::new();
        for index in 0..100 {
            let mut branch = guard.clone();
            append!(branch, " && (kind === {index})");
            guards.push(branch);
            append!(guard, " && !(kind === {index})");
        }
        guards.push(guard);
        let refs: Vec<&str> = guards.iter().map(String::as_str).collect();
        let ts = emit(&refs);
        assert_eq!(ts.matches("(parent)").count(), 1, "{ts}");
        assert_eq!(ts.matches("kind ===").count(), 100, "{ts}");
        assert_eq!(ts.matches("void check").count(), 101, "{ts}");
        assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
    }

    #[test]
    fn unrelated_guards_open_separate_blocks() {
        let ts = emit(&["(a)", "(b)"]);
        assert_eq!(ts.matches("else").count(), 0, "{ts}");
        assert_eq!(ts.matches("if (").count(), 2, "{ts}");
    }

    #[test]
    fn comment_lines_between_blocks_do_not_stop_folding() {
        let mut ts = String::from("");
        for guard in ["(a)", "(a)", "!(a) && (b)"] {
            ts.push_str("  // @vize-map: prop -> 1:2\n");
            append_chained_vif_guard_open(&mut ts, "  ", guard, "Inference-only guard");
            ts.push_str("    void check;\n");
            append_chained_vif_guard_close(&mut ts, "  ");
        }
        assert_eq!(ts.matches("if (").count(), 2, "{ts}");
        assert_eq!(ts.matches("else if ((b))").count(), 1, "{ts}");
        assert_eq!(ts.matches("@vize-map").count(), 3, "{ts}");
        assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
    }

    #[test]
    fn a_block_after_other_output_is_not_folded() {
        let mut ts = String::from("");
        append_chained_vif_guard_open(&mut ts, "  ", "(a)", "Inference-only guard");
        append_chained_vif_guard_close(&mut ts, "  ");
        ts.push_str("  const between = 1;\n");
        append_chained_vif_guard_open(&mut ts, "  ", "!(a) && (b)", "Inference-only guard");
        append_chained_vif_guard_close(&mut ts, "  ");
        assert_eq!(ts.matches("else").count(), 0, "{ts}");
    }
}
