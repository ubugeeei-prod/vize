#[cfg(creusot)]
use creusot_std::prelude::*;

/// Insert after earlier equal ranks. Existing rows shift by one up to the
/// first free slot or the end of the window. The latter drops its last row.
///
/// In a window whose free slots all follow its rows, the row lands at the
/// first slot that is free or holds a row it ranks `before`, every earlier
/// slot keeps its row, and every later slot takes its predecessor's.
#[cfg_attr(creusot, requires(forall<a: T, b: T> before.precondition((a, b))))]
#[cfg_attr(creusot, requires(packed((*slots)@)))]
#[cfg_attr(creusot, ensures(exists<at: Int> lands(before, row, (*slots)@, at)
    && shifted(row, (*slots)@, (^slots)@, at)))]
pub(super) fn insert<T: Copy>(slots: &mut [Option<T>], row: T, before: impl Fn(T, T) -> bool) {
    #[cfg(creusot)]
    let old = snapshot!((*slots)@);
    let mut at = 0;
    #[cfg_attr(creusot, invariant(at@ <= slots@.len() && slots@ == *old))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < at@ ==> exists<rank: T>
        slots@[i] == Some(rank) && before.postcondition((row, rank), false)))]
    #[cfg_attr(creusot, variant(slots@.len() - at@))]
    while let Some(&slot) = slots.get(at) {
        match slot {
            Some(old) if !before(row, old) => at += 1,
            _ => break,
        }
    }
    #[cfg(creusot)]
    let found = snapshot!(at);
    #[cfg(creusot)]
    proof_assert!(lands(before, row, *old, found@));
    let mut displaced = Some(row);
    #[cfg_attr(creusot, invariant(found@ <= at@ && at@ <= slots@.len()))]
    #[cfg_attr(creusot, invariant(slots@.len() == old.len()))]
    #[cfg_attr(creusot, invariant(displaced == if at == *found { Some(row) } else { old[at@ - 1] }))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < slots@.len() ==> slots@[i] ==
        if i < found@ || at@ <= i { old[i] } else if i == found@ { Some(row) } else { old[i - 1] }))]
    #[cfg_attr(creusot, variant(slots@.len() - at@))]
    while at < slots.len() {
        if let Some(slot) = slots.get_mut(at) {
            core::mem::swap(slot, &mut displaced);
        }
        at += 1;
        if displaced.is_none() {
            #[cfg(creusot)]
            proof_assert!(forall<i: Int> at@ - 1 <= i && i < old.len() ==> old[i] == None);
            break;
        }
    }
    #[cfg(creusot)]
    proof_assert!(shifted(row, *old, slots@, found@));
}

/// Every free slot follows every occupied one.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn packed<T>(slots: Seq<Option<T>>) -> bool {
    pearlite! { forall<i: Int, j: Int> 0 <= i && i <= j && j < slots.len() && slots[i] == None
    ==> slots[j] == None }
}

/// Slot `at` is the first one that is free or holds a row that `row` ranks
/// before; `at` is the capacity when there is none.
#[cfg(creusot)]
#[logic(open(crate), prophetic)]
pub(crate) fn lands<T, F: Fn(T, T) -> bool>(
    before: F,
    row: T,
    slots: Seq<Option<T>>,
    at: Int,
) -> bool {
    pearlite! {
        0 <= at && at <= slots.len()
            && (forall<i: Int> 0 <= i && i < at ==> exists<rank: T>
                slots[i] == Some(rank) && before.postcondition((row, rank), false))
            && (at < slots.len() ==> match slots[at] {
                None => true,
                Some(rank) => before.postcondition((row, rank), true),
            })
    }
}

/// `new` is `old` with `row` inserted at `at` and its last slot dropped.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn shifted<T>(row: T, old: Seq<Option<T>>, new: Seq<Option<T>>, at: Int) -> bool {
    pearlite! {
        new.len() == old.len()
            && forall<i: Int> 0 <= i && i < new.len() ==> new[i] ==
                if i < at { old[i] } else if i == at { Some(row) } else { old[i - 1] }
    }
}
