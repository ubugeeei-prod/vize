//! The if- and for-half comparison bodies of [`super::compare`]: the
//! series-1 chain check and the series-2 binding-surface check, split
//! out of `mod.rs` under the 350-line source budget (the slot half
//! lives in [`super::slots`], the text half in [`super::text`]).

use super::Counters;
use super::l2_lane::{L2Chain, L2For, L2Key};
use super::old_lane::{self, OldChain, OldFor, OldKey};

/// One divergence panic, with everything needed to investigate.
macro_rules! diverged {
    ($name:expr, $source:expr, $old:expr, $s2:expr, $($why:tt)+) => {
        panic!(
            "TS-25 divergence [{}]: {}\ntemplate:\n{}\nlegacy projection: {:#?}\ns2 projection: {:#?}",
            $name, format_args!($($why)+), $source, $old, $s2
        )
    };
}

pub fn check(name: &str, source: &str, old: &[OldChain], s2: &[L2Chain], counters: &mut Counters) {
    if old.len() != s2.len() {
        diverged!(
            name,
            source,
            old,
            s2,
            "chain count {} vs {}",
            old.len(),
            s2.len()
        );
    }
    for (chain_index, (old_chain, l2_chain)) in old.iter().zip(s2).enumerate() {
        if old_chain.branches.len() != l2_chain.branches.len() {
            diverged!(name, source, old, s2, "chain {chain_index} branch count");
        }
        counters.if_ops += 1;
        for (old_branch, l2_branch) in old_chain.branches.iter().zip(&l2_chain.branches) {
            counters.branches += 1;
            match (&old_branch.condition, &l2_branch.condition) {
                (None, None) => {}
                (Some(None), Some(_)) => counters.conditions_compound += 1,
                (Some(Some(old_text)), Some(l2_text)) if old_text == l2_text => {}
                _ => diverged!(
                    name,
                    source,
                    old,
                    s2,
                    "chain {chain_index} condition {:?} vs {:?}",
                    old_branch.condition,
                    l2_branch.condition
                ),
            }
            // Key comparison, wrapper and carrier alike (series 5
            // closed the dynamic-key and outlet-key classes; the
            // wrapper key rides the lowering's capture channel). The
            // counted classes: a legacy dynamic-argument `:[key]`
            // (the arg-content quirk L2 now mirrors on ordinary branch
            // carriers, while wrapper residuals still only count) and a
            // legacy compound key rebuild.
            let wrapper = old_branch.template_if;
            match (&old_branch.key, &l2_branch.key) {
                (OldKey::None, None) => {}
                (
                    OldKey::Dynamic {
                        text: Some(old_text),
                        dynamic_arg: true,
                    },
                    Some(L2Key::Dynamic(l2_text)),
                ) if old_text == l2_text => counters.keys_dynamic_arg += 1,
                (
                    OldKey::Dynamic {
                        dynamic_arg: true, ..
                    },
                    None,
                ) if wrapper => counters.keys_dynamic_arg += 1,
                (OldKey::Static(old_value), Some(L2Key::Static(l2_value)))
                    if old_value == l2_value =>
                {
                    if wrapper {
                        counters.keys_wrapper += 1;
                    } else {
                        counters.keys_static += 1;
                    }
                }
                (
                    OldKey::Dynamic {
                        text: Some(old_text),
                        dynamic_arg: false,
                    },
                    Some(L2Key::Dynamic(l2_text)),
                ) if old_text == l2_text => {
                    if wrapper {
                        counters.keys_wrapper += 1;
                    } else {
                        counters.keys_dynamic += 1;
                    }
                }
                (
                    OldKey::Dynamic {
                        text: None,
                        dynamic_arg: false,
                    },
                    Some(L2Key::Dynamic(_)),
                ) => counters.keys_compound += 1,
                _ => diverged!(
                    name,
                    source,
                    old,
                    s2,
                    "chain {chain_index} key {:?} vs {:?}",
                    old_branch.key,
                    l2_branch.key
                ),
            }
        }
    }
}

/// The series-2 half: every for's binding surface, in document order.
pub fn check_fors(name: &str, source: &str, old: &[OldFor], s2: &[L2For], counters: &mut Counters) {
    if old.len() != s2.len() {
        diverged!(
            name,
            source,
            old,
            s2,
            "for count {} vs {}",
            old.len(),
            s2.len()
        );
    }
    for (for_index, (old_for, l2_for)) in old.iter().zip(s2).enumerate() {
        counters.for_ops += 1;
        match &old_for.source {
            None => counters.for_compound += 1,
            Some(old_text) if *old_text == l2_for.source => {}
            Some(_) => diverged!(
                name,
                source,
                old,
                s2,
                "for {for_index} source {:?} vs {:?}",
                old_for.source,
                l2_for.source
            ),
        }
        /// What one alias position's comparison found.
        enum Alias {
            BothAbsent,
            Compound,
            Compared,
        }
        let alias = |position: &str,
                     old_alias: &Option<old_lane::OldText>,
                     l2_alias: &Option<vize_l0::String>| {
            match (old_alias, l2_alias) {
                (None, None) => Alias::BothAbsent,
                (Some(None), Some(_)) => Alias::Compound,
                (Some(Some(old_text)), Some(l2_text)) if old_text == l2_text => Alias::Compared,
                _ => diverged!(
                    name,
                    source,
                    old,
                    s2,
                    "for {for_index} {position} {:?} vs {:?}",
                    old_alias,
                    l2_alias
                ),
            }
        };
        match alias("value", &old_for.value, &l2_for.value) {
            Alias::BothAbsent => counters.for_values_absent += 1,
            Alias::Compound => counters.for_compound += 1,
            Alias::Compared => counters.for_values += 1,
        }
        match alias("key", &old_for.key, &l2_for.key) {
            Alias::BothAbsent => {}
            Alias::Compound => counters.for_compound += 1,
            Alias::Compared => counters.for_keys += 1,
        }
        match alias("index", &old_for.index, &l2_for.index) {
            Alias::BothAbsent => {}
            Alias::Compound => counters.for_compound += 1,
            Alias::Compared => counters.for_indexes += 1,
        }
    }
}
