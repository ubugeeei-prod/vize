//! Creusot proves the fusion grouping on the shipped scanner for plans of
//! any length and arbitrary 64-bit preservation masks. This file is the
//! library root of `tests/formal/creusot/fusion` and is not part of
//! `vize_l0`'s module tree: `prefix` below is the module the product
//! compiles, found by ordinary module discovery.
//!
//! The grouping rule is the one `fusion.rs` documents: walk the passes in
//! order, a barrier sits alone, adjacent fusable passes collect into one
//! group, and a group preserves the intersection of its members' masks.
//! `prefix::next_group` states the rule for the first group of any slice and
//! any classifier. The law below drives it over a whole plan, as the product
//! cursor does, and checks that the groups partition the plan.
mod prefix;

pub use prefix::{
    first_group, keeps, meet, meet_reads_the_first_classes, the_meet_keeps_exactly_the_common_bits,
};

use creusot_std::prelude::*;

/// A pass modelled by its class: whether it is fusable, and its mask.
#[ensures(result == *pass)]
pub fn class(pass: &(bool, u64)) -> (bool, u64) {
    *pass
}

/// The groups `next_group` cuts from a plan partition it in order: each
/// group starts where the previous one ended, is nonempty, is a lone barrier
/// exactly when its first pass is not fusable, otherwise holds every
/// fusable pass up to the next barrier, and preserves the intersection of
/// its members' masks.
pub fn groups_partition_the_plan_and_preserve_exactly_the_common_analyses(passes: &[(bool, u64)]) {
    let mut start = 0;
    #[invariant(start@ <= passes@.len())]
    #[variant(passes@.len() - start@)]
    loop {
        let rest = &passes[start..];
        proof_assert!(rest@.len() == passes@.len() - start@);
        proof_assert!(forall<k: Int> 0 <= k && k < rest@.len() ==> rest[k] == passes[start@ + k]);
        let Some((len, barrier, preserved)) = prefix::next_group(rest, class) else {
            break;
        };
        proof_assert!(forall<classes: Seq<(bool, u64)>>
            first_group(rest@.len(), classes, len@, barrier, preserved)
            && (forall<i: Int> 0 <= i && i < classes.len() ==> classes[i] == rest[i])
            ==> {
                let end = start@ + len@;
                0 < len@ && end <= passes@.len()
                    && barrier == !passes[start@].0
                    && (barrier ==> len@ == 1)
                    && (forall<k: Int> 0 <= k && k < len@ ==> passes[start@ + k].0 == !barrier)
                    && (!barrier && end < passes@.len() ==> !passes[end].0)
                    && {
                        meet_reads_the_first_classes(classes, passes@.subsequence(start@, end), len@);
                        preserved == meet(passes@.subsequence(start@, end), len@)
                    }
            });
        let end = start + len;
        proof_assert!(0 < len@ && end@ <= passes@.len());
        proof_assert!(barrier == !passes[start@].0 && (barrier ==> len@ == 1));
        proof_assert!(forall<k: Int> 0 <= k && k < len@ ==> passes[start@ + k].0 == !barrier);
        proof_assert!(!barrier && end@ < passes@.len() ==> !passes[end@].0);
        proof_assert!(preserved == meet(passes@.subsequence(start@, end@), len@));
        proof_assert!(forall<bit: u64> bit < 64u64 ==> {
            the_meet_keeps_exactly_the_common_bits(passes@.subsequence(start@, end@), len@, bit);
            keeps(preserved, bit)
                == forall<k: Int> 0 <= k && k < len@ ==> keeps(passes[start@ + k].1, bit)
        });
        start = end;
    }
    proof_assert!(start@ == passes@.len());
}
