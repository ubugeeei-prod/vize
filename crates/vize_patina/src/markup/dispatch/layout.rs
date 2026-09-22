//! [`MarkupRuleLayout`]: a fused rule set's dispatch table over registry
//! indices. It depends only on the registry, so a lint pass lays it out once
//! and each template only resolves it ([`MarkupRuleLayout::resolve`]) — no
//! subscription is re-read and no table re-sorted per template.

use super::{HOOK_COUNT, MarkupHooks, MarkupRuleSet};
use crate::rule::Rule;

/// Per hook, the registry indices of the rules subscribed to it, in
/// registration order.
#[derive(Debug, Clone, Default)]
pub struct MarkupRuleLayout {
    table: Vec<usize>,
    /// `table[bounds[h]..bounds[h + 1]]` subscribe to hook `h`.
    bounds: [usize; HOOK_COUNT + 1],
}

impl MarkupRuleLayout {
    /// Lay out `(registry index, subscription)` pairs, given in registration
    /// order.
    pub fn new(subscriptions: &[(usize, MarkupHooks)]) -> Self {
        let mut layout = Self::default();
        for (hook_index, hook) in MarkupHooks::EACH.into_iter().enumerate() {
            layout.table.extend(
                subscriptions
                    .iter()
                    .filter(|(_, hooks)| hooks.contains(hook))
                    .map(|&(index, _)| index),
            );
            layout.bounds[hook_index + 1] = layout.table.len();
        }
        layout
    }

    /// The rule set over the first `rules.len()` registered rules.
    pub fn resolve<'r>(
        &self,
        rules: &'r [Box<dyn Rule>],
        names: &[&'static str],
    ) -> MarkupRuleSet<'r> {
        let mut entries = Vec::with_capacity(self.table.len());
        let mut bounds = [0u16; HOOK_COUNT + 1];
        for hook in 0..HOOK_COUNT {
            for &index in &self.table[self.bounds[hook]..self.bounds[hook + 1]] {
                if let Some(rule) = rules.get(index).and_then(|rule| rule.as_markup_rule()) {
                    entries.push((rule, names[index]));
                }
            }
            bounds[hook + 1] = entries.len() as u16;
        }
        MarkupRuleSet { entries, bounds }
    }
}
