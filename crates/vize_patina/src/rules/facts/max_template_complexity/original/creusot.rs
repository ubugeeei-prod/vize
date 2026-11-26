//! Creusot proves that the complexity rule's label window keeps exactly the
//! first rows of the stable ranking, on its shipped source (`window.rs`).
//! This file is the library root of `tests/formal/creusot/template-labels`
//! and is not part of `vize_patina`'s module tree: the module below is the
//! one the product compiles, found by ordinary module discovery.
//!
//! The laws follow tests/formal/template-lint/Selection.lean. `insert_full`
//! is its unlimited stable insertion (`insertFull`) and `ranking` its fold.
//! `insert` carries its contract in the shipped source; the laws below are
//! stated over it. Unlike the Kani harness they replace, which fixed five
//! slots and `u8` scores, they hold for every capacity, every number of
//! rows and any comparison that decides each pair: `rank` is the ghost
//! relation the comparison computes.
//! See docs/davinci/decisions/2026-11-17-creusot.md.

mod window;

use creusot_std::{logic::Mapping, prelude::*};
use window::{insert, lands, packed, shifted};

/// `rank` is a pair's ranking, the comparison answers every pair, and it
/// answers with `rank`.
#[logic(open(crate), prophetic)]
pub fn computes<T, F: Fn(T, T) -> bool>(before: F, rank: Mapping<(T, T), bool>) -> bool {
    pearlite! {
        forall<a: T, b: T> before.precondition((a, b))
            && forall<r: bool> before.postcondition((a, b), r) ==> r == rank.get((a, b))
    }
}

/// The unlimited stable insertion: the row goes after every row it does not
/// rank before (`insertFull`).
#[logic(open(crate))]
#[variant(rows.len())]
pub fn insert_full<T>(rank: Mapping<(T, T), bool>, row: T, rows: Seq<T>) -> Seq<T> {
    pearlite! {
        if rows.len() == 0 {
            Seq::singleton(row)
        } else if rank.get((row, rows[0])) {
            rows.push_front(row)
        } else {
            insert_full(rank, row, rows.tail()).push_front(rows[0])
        }
    }
}

/// The stable ranking of `rows`: each row inserted in input order.
#[logic(open(crate))]
#[variant(rows.len())]
pub fn ranking<T>(rank: Mapping<(T, T), bool>, rows: Seq<T>) -> Seq<T> {
    pearlite! {
        if rows.len() == 0 {
            Seq::empty()
        } else {
            insert_full(rank, rows[rows.len() - 1], ranking(rank, rows.subsequence(0, rows.len() - 1)))
        }
    }
}

/// The window holds `rows` in rank order, then free slots.
#[logic(open(crate))]
pub fn holds<T>(slots: Seq<Option<T>>, rows: Seq<T>) -> bool {
    pearlite! {
        rows.len() <= slots.len()
            && (forall<i: Int> 0 <= i && i < rows.len() ==> slots[i] == Some(rows[i]))
            && (forall<i: Int> rows.len() <= i && i < slots.len() ==> slots[i] == None)
    }
}

/// The first `capacity` entries of `rows`.
#[logic(open(crate))]
pub fn first<T>(rows: Seq<T>, capacity: Int) -> Seq<T> {
    pearlite! { rows.subsequence(0, if rows.len() <= capacity { rows.len() } else { capacity }) }
}

/// The stable insertion adds exactly one row.
#[logic]
#[variant(rows.len())]
#[ensures(insert_full(rank, row, rows).len() == rows.len() + 1)]
pub fn insertion_adds_one_row<T>(rank: Mapping<(T, T), bool>, row: T, rows: Seq<T>) {
    pearlite! {
        if rows.len() > 0 && !rank.get((row, rows[0])) {
            insertion_adds_one_row(rank, row, rows.tail())
        }
    }
}

/// Rows the new row does not rank before keep their places, and once it
/// ranks before one, it takes that place and the rest shift by one.
#[logic]
#[variant(at)]
#[requires(0 <= at && at <= rows.len())]
#[requires(forall<i: Int> 0 <= i && i < at ==> !rank.get((row, rows[i])))]
#[ensures(insert_full(rank, row, rows).len() == rows.len() + 1)]
#[ensures(forall<i: Int> 0 <= i && i < at ==> insert_full(rank, row, rows)[i] == rows[i])]
#[ensures(at == rows.len() || rank.get((row, rows[at])) ==>
    insert_full(rank, row, rows)[at] == row
        && forall<i: Int> at < i && i <= rows.len() ==> insert_full(rank, row, rows)[i] == rows[i - 1])]
pub fn insertion_splits_at_the_first_row_it_ranks_before<T>(
    rank: Mapping<(T, T), bool>,
    row: T,
    rows: Seq<T>,
    at: Int,
) {
    pearlite! {
        if at > 0 {
            insertion_splits_at_the_first_row_it_ranks_before(rank, row, rows.tail(), at - 1)
        } else {
            insertion_adds_one_row(rank, row, rows)
        }
    }
}

/// `lands` with the comparison's answers read from `rank`.
#[logic(open(crate))]
pub fn ranked_landing<T>(
    rank: Mapping<(T, T), bool>,
    row: T,
    slots: Seq<Option<T>>,
    at: Int,
) -> bool {
    pearlite! {
        0 <= at && at <= slots.len()
            && (forall<i: Int> 0 <= i && i < at ==> exists<old: T>
                slots[i] == Some(old) && !rank.get((row, old)))
            && (at < slots.len() ==> match slots[at] {
                None => true,
                Some(old) => rank.get((row, old)),
            })
    }
}

/// A row lands among the kept rows: every row before it ranks after it, the
/// kept row it lands on ranks after it, and it lands past the kept rows only
/// when the ranking has no more rows.
#[logic]
#[requires(holds(old, first(rows, old.len())))]
#[requires(ranked_landing(rank, row, old, at))]
#[ensures(at <= first(rows, old.len()).len())]
#[ensures(forall<i: Int> 0 <= i && i < at ==> !rank.get((row, rows[i])))]
#[ensures(at < first(rows, old.len()).len() ==> rank.get((row, rows[at])))]
#[ensures(at == first(rows, old.len()).len() && at < old.len() ==> at == rows.len())]
pub fn a_row_lands_among_the_kept_rows<T>(
    rank: Mapping<(T, T), bool>,
    row: T,
    rows: Seq<T>,
    old: Seq<Option<T>>,
    at: Int,
) {
    pearlite! {
        {
            let kept = first(rows, old.len()).len();
            proof_assert!(kept == if rows.len() <= old.len() { rows.len() } else { old.len() });
            proof_assert!(forall<i: Int> 0 <= i && i < kept ==> old[i] == Some(rows[i]));
            proof_assert!(forall<i: Int> kept <= i && i < old.len() ==> old[i] == None);
            proof_assert!(kept < at ==> old[kept] != None);
            proof_assert!(at <= kept);
            proof_assert!(forall<i: Int> 0 <= i && i < at ==> old[i] == Some(rows[i]) && !rank.get((row, rows[i])))
        }
    }
}

/// The window step of `insert_prefix`: landing and shifting a row into the
/// first rows of a ranking leaves the first rows of the ranking with the row
/// inserted.
#[logic]
#[requires(holds(old, first(rows, old.len())))]
#[requires(ranked_landing(rank, row, old, at))]
#[requires(shifted(row, old, new, at))]
#[ensures(holds(new, first(insert_full(rank, row, rows), old.len())))]
pub fn a_landed_row_extends_the_ranking_prefix<T>(
    rank: Mapping<(T, T), bool>,
    row: T,
    rows: Seq<T>,
    old: Seq<Option<T>>,
    new: Seq<Option<T>>,
    at: Int,
) {
    pearlite! {
        {
            let _ = a_row_lands_among_the_kept_rows(rank, row, rows, old, at);
            let _ = insertion_splits_at_the_first_row_it_ranks_before(rank, row, rows, at);
            let ins = insert_full(rank, row, rows);
            let size = first(ins, old.len()).len();
            proof_assert!(size == if rows.len() + 1 <= old.len() { rows.len() + 1 } else { old.len() });
            proof_assert!(forall<i: Int> 0 <= i && i < at ==> new[i] == Some(ins[i]));
            proof_assert!(at < size ==> new[at] == Some(ins[at]));
            proof_assert!(forall<i: Int> at < i && i < size ==> new[i] == Some(ins[i]));
            proof_assert!(forall<i: Int> size <= i && i < new.len() ==> new[i] == None)
        }
    }
}

/// Inserting into the window that holds the first rows of any ranking
/// leaves it holding the first rows of that ranking with the row inserted
/// (`insert_prefix`): for every capacity and any number of rows beyond it.
#[requires(computes(before, *rank))]
#[requires(holds(slots@, first(*rows, slots@.len())))]
#[ensures(holds((^slots)@, first(insert_full(*rank, row, *rows), slots@.len())))]
#[ensures((^slots)@.len() == slots@.len())]
pub fn every_displaced_label_is_the_reference_prefix_entry<T: Copy, F: Fn(T, T) -> bool>(
    slots: &mut [Option<T>],
    row: T,
    before: F,
    rank: Snapshot<Mapping<(T, T), bool>>,
    rows: Snapshot<Seq<T>>,
) {
    let old = snapshot!((*slots)@);
    let comparison = snapshot!(before);
    proof_assert!(packed(*old));
    insert(slots, row, before);
    proof_assert!(forall<at: Int> lands(*comparison, row, *old, at) ==> ranked_landing(*rank, row, *old, at));
    proof_assert!(forall<at: Int> ranked_landing(*rank, row, *old, at) && shifted(row, *old, slots@, at)
        ==> a_landed_row_extends_the_ranking_prefix(*rank, row, *rows, *old, slots@, at) == ());
}

/// After inserting every row into a free window, it holds the first rows
/// of their stable ranking (`fold_prefix`), so the rule labels exactly the
/// top rows the unlimited ranking would.
#[requires(computes(before, *rank))]
#[requires(forall<i: Int> 0 <= i && i < slots@.len() ==> slots@[i] == None)]
#[ensures(holds((^slots)@, first(ranking(*rank, rows@), slots@.len())))]
pub fn the_window_is_the_prefix_of_the_stable_ranking<T: Copy, F: Fn(T, T) -> bool + Copy>(
    slots: &mut [Option<T>],
    rows: &[T],
    before: F,
    rank: Snapshot<Mapping<(T, T), bool>>,
) {
    let capacity = snapshot!(slots@.len());
    let mut seen = 0;
    #[invariant(seen@ <= rows@.len() && slots@.len() == *capacity)]
    #[invariant(holds(slots@, first(ranking(*rank, rows@.subsequence(0, seen@)), *capacity)))]
    #[variant(rows@.len() - seen@)]
    while seen < rows.len() {
        every_displaced_label_is_the_reference_prefix_entry(
            slots,
            rows[seen],
            before,
            rank,
            snapshot!(ranking(*rank, rows@.subsequence(0, seen@))),
        );
        seen += 1;
        proof_assert!(rows@.subsequence(0, seen@).subsequence(0, seen@ - 1) == rows@.subsequence(0, seen@ - 1));
        proof_assert!(rows@.subsequence(0, seen@)[seen@ - 1] == rows@[seen@ - 1]);
        proof_assert!(ranking(*rank, rows@.subsequence(0, seen@))
            == insert_full(*rank, rows@[seen@ - 1], ranking(*rank, rows@.subsequence(0, seen@ - 1))));
    }
    proof_assert!(rows@.subsequence(0, rows@.len()) == rows@);
    proof_assert!(ranking(*rank, rows@.subsequence(0, rows@.len())) == ranking(*rank, rows@));
}
