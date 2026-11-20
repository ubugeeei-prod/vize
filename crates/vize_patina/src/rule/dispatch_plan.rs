//! Precomputed legacy-template callback dispatch.

use super::{Rule, RuleHooks};
use vize_l0::SmallVec;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndexedHookMask {
    pub(crate) words: SmallVec<[u64; 2]>,
}

impl IndexedHookMask {
    fn insert(&mut self, rank: usize) {
        let word = rank / u64::BITS as usize;
        while self.words.len() <= word {
            self.words.push(0);
        }
        if let Some(bits) = self.words.get_mut(word) {
            *bits |= 1 << (rank % u64::BITS as usize);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndexedRulePlans {
    pub(crate) template: IndexedHookMask,
    pub(crate) element: IndexedHookMask,
    pub(crate) exit: IndexedHookMask,
    pub(crate) directive: IndexedHookMask,
    pub(crate) for_node: IndexedHookMask,
    pub(crate) if_node: IndexedHookMask,
    pub(crate) interpolation: IndexedHookMask,
}

impl IndexedRulePlans {
    pub(crate) fn new(rules: &[Box<dyn Rule>], indices: &[usize]) -> Self {
        let mut plans = Self::default();
        for &rank in indices {
            let Some(rule) = rules.get(rank) else {
                continue;
            };
            let hooks = rule.template_hooks();
            if hooks == RuleHooks::NONE {
                continue;
            }
            for (hook, mask) in [
                (RuleHooks::TEMPLATE, &mut plans.template),
                (RuleHooks::ELEMENT, &mut plans.element),
                (RuleHooks::EXIT, &mut plans.exit),
                (RuleHooks::DIRECTIVE, &mut plans.directive),
                (RuleHooks::FOR, &mut plans.for_node),
                (RuleHooks::IF, &mut plans.if_node),
                (RuleHooks::INTERPOLATION, &mut plans.interpolation),
            ] {
                if hooks.contains(hook) {
                    mask.insert(rank);
                }
            }
        }
        plans
    }
}

#[derive(Clone)]
pub(crate) struct LegacyTemplatePlan {
    pub(crate) indices: Vec<usize>,
    pub(crate) hooks: IndexedRulePlans,
}

impl LegacyTemplatePlan {
    pub(crate) fn new(rules: &[Box<dyn Rule>], count: usize, facade_ranks: &[usize]) -> Self {
        let count = count.min(rules.len());
        let mut indices = Vec::with_capacity(count.saturating_sub(facade_ranks.len()));
        let mut facade_index = 0;
        for rank in 0..count {
            while facade_ranks
                .get(facade_index)
                .is_some_and(|facade_rank| *facade_rank < rank)
            {
                facade_index += 1;
            }
            if facade_ranks
                .get(facade_index)
                .is_some_and(|facade_rank| *facade_rank == rank)
            {
                facade_index += 1;
            } else {
                indices.push(rank);
            }
        }
        let hooks = IndexedRulePlans::new(rules, &indices);
        Self { indices, hooks }
    }
}
