//! The template lanes' dispatch plan: which registered rule runs in which lane
//! and, for a markup rule, at which hooks. It depends only on the registry, so
//! a [`crate::Linter`] computes it once (and drops it whenever its registry
//! changes) instead of asking every rule again on every template.

use crate::markup::{MarkupRuleLayout, MarkupRuleSet};
use crate::rule::{Rule, RuleRegistry};

/// Registry indices per lane, in registration order.
#[derive(Debug, Clone, Default)]
pub(crate) struct LanePlan {
    directive: Vec<usize>,
    markup: MarkupRuleLayout,
}

impl LanePlan {
    pub(crate) fn new(registry: &RuleRegistry) -> Self {
        let mut directive = Vec::new();
        let mut markup = Vec::new();
        for (index, rule) in registry.rules().iter().enumerate() {
            match rule.as_markup_rule() {
                Some(body) if rule.markup_on_templates() => markup.push((index, body.hooks())),
                _ => directive.push(index),
            }
        }
        Self {
            directive,
            markup: MarkupRuleLayout::new(&markup),
        }
    }

    /// The directive-lane rules among the first `rules.len()` registered.
    pub(crate) fn directive<'r>(
        &self,
        rules: &'r [Box<dyn Rule>],
        names: &[&'static str],
    ) -> Vec<(&'r dyn Rule, &'static str)> {
        let mut active = Vec::with_capacity(self.directive.len());
        active.extend(
            self.directive
                .iter()
                .take_while(|&&index| index < rules.len())
                .map(|&index| (&*rules[index], names[index])),
        );
        active
    }

    /// The markup-lane rules among the first `rules.len()` registered.
    pub(crate) fn markup<'r>(
        &self,
        rules: &'r [Box<dyn Rule>],
        names: &[&'static str],
    ) -> MarkupRuleSet<'r> {
        self.markup.resolve(rules, names)
    }
}
