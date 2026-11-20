//! Specialize editor bookkeeping outside the ordinary per-rule hot loop.

use crate::context::{CallbackSite, LintContext};
use crate::rule::{IndexedHookMask, Rule};

use super::{LintVisitor, RuleSelection};

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    #[inline]
    pub(super) fn callback_rules(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        selection: RuleSelection<'_>,
        indexed_mask: &IndexedHookMask,
        site: CallbackSite,
        visit: impl FnMut(usize, &dyn Rule, &mut LintContext<'a>),
    ) {
        if ctx.callback_order_enabled() {
            ctx.record_callback_site(site);
            Self::callback_rules_inner::<true>(ctx, rules, names, selection, indexed_mask, visit);
        } else {
            Self::callback_rules_inner::<false>(ctx, rules, names, selection, indexed_mask, visit);
        }
    }

    #[inline]
    fn callback_rules_inner<const ORDERED: bool>(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        selection: RuleSelection<'_>,
        indexed_mask: &IndexedHookMask,
        mut visit: impl FnMut(usize, &dyn Rule, &mut LintContext<'a>),
    ) {
        match selection {
            RuleSelection::All => {
                for (index, (rule, name)) in rules.iter().zip(names.iter().copied()).enumerate() {
                    if ORDERED {
                        ctx.set_callback_rule(index);
                    }
                    ctx.current_rule = name;
                    visit(index, rule.as_ref(), ctx);
                }
            }
            RuleSelection::Mask(mask) => {
                for (index, (rule, name)) in rules.iter().zip(names.iter().copied()).enumerate() {
                    if !mask.get(index).copied().unwrap_or(false) {
                        continue;
                    }
                    if ORDERED {
                        ctx.set_callback_rule(index);
                    }
                    ctx.current_rule = name;
                    visit(index, rule.as_ref(), ctx);
                }
            }
            RuleSelection::Indices(_) => {
                for (word_index, &word) in indexed_mask.words.iter().enumerate() {
                    let mut remaining = word;
                    while remaining != 0 {
                        let bit = remaining.trailing_zeros() as usize;
                        remaining &= remaining - 1;
                        let index = word_index * u64::BITS as usize + bit;
                        let Some((rule, name)) = rules.get(index).zip(names.get(index).copied())
                        else {
                            continue;
                        };

                        if ORDERED {
                            ctx.set_callback_rule(index);
                        }
                        ctx.current_rule = name;
                        visit(index, rule.as_ref(), ctx);
                    }
                }
            }
        }
    }
}
