//! The reduction driver (P3-14): llvm-reduce's shape — a dumb, deterministic
//! loop over an IR-aware deletion vocabulary, with a sovereign
//! interestingness predicate deciding every step.
//!
//! Each round enumerates the current artifact's S1 candidates (pre-order,
//! largest first) and runs ddmin-style chunking over them: try deleting a
//! chunk of `size` consecutive candidates; keep the deletion when the
//! predicate still holds (and re-enumerate, since every range moved);
//! otherwise move to the next chunk; halve `size` when a pass over the list
//! kept nothing. A round that keeps nothing at size 1 is a fixpoint:
//! **1-minimal** with respect to the vocabulary — no single remaining node
//! or attribute can be deleted without losing the oracle. The loop is
//! deterministic given a deterministic predicate, so a reduction is a
//! reproducible artifact, pinnable by exact tests.

use std::ops::Range;

use vize_s0::String;

use super::vocabulary::{candidates, delete};

/// Why a reduction did not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReduceError {
    /// The input itself does not satisfy the oracle: there is nothing to
    /// preserve.
    NotInteresting,
}

/// A finished reduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reduction {
    /// The reduced artifact; still satisfies the oracle.
    pub(crate) text: String,
    /// Predicate evaluations spent, the initial check included.
    pub(crate) runs: usize,
    /// Whether the run budget ran out before the fixpoint.
    pub(crate) exhausted: bool,
}

/// Reduce `input` while `interesting` holds, spending at most `max_runs`
/// predicate evaluations.
pub(crate) fn reduce(
    input: &str,
    max_runs: usize,
    interesting: &mut dyn FnMut(&str) -> bool,
) -> Result<Reduction, ReduceError> {
    let mut runs = 1;
    if !interesting(input) {
        return Err(ReduceError::NotInteresting);
    }
    let mut current = String::from(input);
    let mut list: Vec<Range<usize>> = candidates(&current);
    let mut size = list.len().max(1);
    loop {
        let mut kept = false;
        let mut index = 0;
        while index < list.len() {
            if runs >= max_runs {
                return Ok(Reduction {
                    text: current,
                    runs,
                    exhausted: true,
                });
            }
            let chunk = &list[index..(index + size).min(list.len())];
            let attempt = delete(&current, chunk);
            if attempt.len() < current.len() {
                runs += 1;
                if interesting(&attempt) {
                    current = attempt;
                    list = candidates(&current);
                    kept = true;
                    continue;
                }
            }
            index += size;
        }
        if size == 1 && !kept {
            return Ok(Reduction {
                text: current,
                runs,
                exhausted: false,
            });
        }
        if !kept {
            size = (size / 2).max(1);
        }
    }
}
