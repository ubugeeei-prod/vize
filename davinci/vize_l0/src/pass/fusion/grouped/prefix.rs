//! Maximal prefix of a group, with the intersection of its analysis masks.
//! The verifier instantiates this same scan with arbitrary masks and barriers.
//!
//! Creusot proves `next_group` against the grouping rule of `fusion.rs` for
//! any slice and any classifier (`tests/formal/creusot/fusion`, root
//! `creusot.rs`): a barrier sits alone, a fusable pass collects every
//! following fusable pass, and the group preserves the intersection of its
//! members' masks.
#[cfg(creusot)]
use creusot_std::prelude::*;

/// The intersection of the first `len` masks of `classes`.
#[cfg(creusot)]
#[logic]
#[variant(len)]
#[requires(1 <= len && len <= classes.len())]
pub fn meet(classes: Seq<(bool, u64)>, len: Int) -> u64 {
    pearlite! {
        if len == 1 { classes[0].1 } else { meet(classes, len - 1) & classes[len - 1].1 }
    }
}

/// The grouping rule for the first group of `passes`, whose classes are
/// `classes`: `len` passes, a barrier exactly when the first pass is not
/// fusable and then alone, otherwise every following fusable pass up to the
/// first one that is not, preserving `meet` of their masks. `classes` holds
/// the class of each group member and, when a pass ends the group, of that
/// pass too.
#[cfg(creusot)]
#[logic(open)]
pub fn first_group(
    passes: Int,
    classes: Seq<(bool, u64)>,
    len: Int,
    barrier: bool,
    mask: u64,
) -> bool {
    pearlite! {
        1 <= len && len <= passes
            && classes.len() == (if barrier || len == passes { len } else { len + 1 })
            && barrier == !classes[0].0
            && (barrier ==> len == 1)
            && (!barrier ==> forall<i: Int> 0 <= i && i < len ==> classes[i].0)
            && (!barrier && len < passes ==> !classes[len].0)
            && mask == meet(classes, len)
    }
}

/// The mask holds analysis `bit`.
#[cfg(creusot)]
#[logic(open)]
pub fn keeps(mask: u64, bit: u64) -> bool {
    pearlite! { mask & (1u64 << bit) != 0u64 }
}

/// An intersection of two masks holds a bit exactly when both do.
#[cfg(creusot)]
#[logic]
#[bitwise_proof]
#[requires(bit < 64u64)]
#[ensures(keeps(left & right, bit) == (keeps(left, bit) && keeps(right, bit)))]
pub fn an_intersection_keeps_the_common_bits(left: u64, right: u64, bit: u64) {}

/// `meet` holds a bit exactly when every one of the first `len` masks does.
#[cfg(creusot)]
#[logic]
#[variant(len)]
#[requires(1 <= len && len <= classes.len())]
#[requires(bit < 64u64)]
#[ensures(keeps(meet(classes, len), bit)
    == forall<i: Int> 0 <= i && i < len ==> keeps(classes[i].1, bit))]
pub fn the_meet_keeps_exactly_the_common_bits(classes: Seq<(bool, u64)>, len: Int, bit: u64) {
    pearlite! {
        if len > 1 {
            an_intersection_keeps_the_common_bits(meet(classes, len - 1), classes[len - 1].1, bit);
            the_meet_keeps_exactly_the_common_bits(classes, len - 1, bit)
        }
    }
}

/// Two class sequences that agree on their first `len` entries meet alike.
#[cfg(creusot)]
#[logic]
#[variant(len)]
#[requires(1 <= len && len <= left.len() && len <= right.len())]
#[requires(forall<i: Int> 0 <= i && i < len ==> left[i] == right[i])]
#[ensures(meet(left, len) == meet(right, len))]
pub fn meet_reads_the_first_classes(left: Seq<(bool, u64)>, right: Seq<(bool, u64)>, len: Int) {
    pearlite! {
        if len > 1 { meet_reads_the_first_classes(left, right, len - 1) }
    }
}

#[inline]
#[cfg_attr(creusot, requires(forall<pass: &T> classify.precondition((pass,))))]
#[cfg_attr(creusot, ensures(match result {
    None => remaining@.len() == 0,
    Some((len, barrier, mask)) => exists<classes: Seq<(bool, u64)>>
        first_group(remaining@.len(), classes, len@, barrier, mask)
            && forall<i: Int> 0 <= i && i < classes.len()
                ==> classify.postcondition((&remaining[i],), classes[i]),
}))]
pub(super) fn next_group<T>(
    remaining: &[T],
    classify: impl Fn(&T) -> (bool, u64),
) -> Option<(usize, bool, u64)> {
    let first = remaining.first()?;
    let tail = remaining.get(1..)?;
    #[cfg(creusot)]
    proof_assert!(forall<j: Int> 0 <= j && j < tail@.len() ==> tail[j] == remaining[j + 1]);
    let (fusable, mut mask) = classify(first);
    #[cfg(creusot)]
    let mut classes = snapshot!(Seq::singleton((fusable, mask)));
    let mut len = 1;
    if fusable {
        #[cfg_attr(creusot, invariant(len@ == produced.len() + 1 && len@ <= remaining@.len()))]
        #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < produced.len()
            ==> produced[j] == &remaining[j + 1]))]
        #[cfg_attr(creusot, invariant(classes.len() == len@))]
        #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < len@ ==> classes[i].0))]
        #[cfg_attr(creusot, invariant(mask == meet(*classes, len@)))]
        #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < classes.len()
            ==> classify.postcondition((&remaining[i],), classes[i])))]
        for pass in tail {
            #[cfg(creusot)]
            proof_assert!(produced.len() == len@ && tail@.len() == remaining@.len() - 1);
            #[cfg(creusot)]
            proof_assert!(produced[len@ - 1] == pass);
            #[cfg(creusot)]
            proof_assert!(tail.to_ref_seq()[len@ - 1] == pass);
            #[cfg(creusot)]
            proof_assert!(*pass == tail[len@ - 1] && *pass == remaining[len@]);
            #[cfg(creusot)]
            proof_assert!(pass == &remaining[len@]);
            let (joins, preserved) = classify(pass);
            #[cfg(creusot)]
            let before = snapshot!(classes);
            #[cfg(creusot)]
            {
                classes = snapshot!(classes.push_back((joins, preserved)));
            }
            if !joins {
                #[cfg(creusot)]
                proof_assert!(first_group(remaining@.len(), *classes, len@, false, mask));
                break;
            }
            #[cfg(creusot)]
            proof_assert!({
                meet_reads_the_first_classes(**before, *classes, len@);
                meet(*classes, len@ + 1) == meet(**before, len@) & preserved
            });
            mask &= preserved;
            len += 1;
        }
    }
    #[cfg(creusot)]
    proof_assert!(first_group(remaining@.len(), *classes, len@, !fusable, mask));
    #[cfg(creusot)]
    proof_assert!(forall<i: Int> 0 <= i && i < classes.len()
        ==> classify.postcondition((&remaining[i],), classes[i]));
    Some((len, !fusable, mask))
}
