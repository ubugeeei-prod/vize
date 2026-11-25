//! Creusot proves the law of the scope table on its shipped source
//! (`rows.rs`): the current row always exists. This file is the library root
//! of `tests/formal/creusot/scope-chain` and is not part of `vize_croquis`'s
//! module tree. The module below is the one the product compiles, found by
//! ordinary module discovery.
//!
//! The law is `Rows`'s type invariant. Creusot assumes it of every table a
//! method receives and proves it of every table a method returns or leaves
//! behind through `&mut`. That covers each constructor, each append, each
//! selection and each in-place edit. Inside `Rows::current` and
//! `Rows::current_mut` the invariant discharges the `requires` that
//! creusot-std gives `<[T]>::get_unchecked` and `get_unchecked_mut`
//! (`ix.in_bounds`), so both unchecked reads are proved in bounds. No `Rows`
//! method has a precondition, so the proof does not depend on how
//! `ScopeChain` or the rest of `vize_croquis` calls it. The function below
//! states the law for every table and every sequence of transitions. See
//! docs/davinci/decisions/2026-11-24-unsafe-model-checking.md.

mod rows;

use rows::Rows;

/// One transition of the scope chain: what `ScopeChain`'s methods do to its
/// table.
pub(crate) enum Transition<T> {
    /// `enter_*`: append a scope and make it current.
    Enter(T),
    /// `set_current` and `exit_scope`: select a scope by id.
    Select(u32),
    /// `reserve_scopes`.
    Reserve(usize),
}

/// Apply `transitions` in order, then take the current row through the
/// unchecked read. The read is proved in bounds after every sequence of
/// transitions from every table, not only from a fresh chain. Creusot proves
/// the body has no failing goal: no transition can need a precondition and no
/// sequence of them can leave the table without its current row.
pub(crate) fn the_current_row_exists_after_every_transition_sequence<T>(
    table: &mut Rows<T>,
    transitions: Vec<Transition<T>>,
) -> &mut T {
    for transition in transitions {
        match transition {
            Transition::Enter(row) => table.push_selected(row),
            Transition::Select(index) => {
                let _ = table.select(index);
            }
            Transition::Reserve(additional) => table.reserve_exact(additional),
        }
    }
    table.current_mut()
}
