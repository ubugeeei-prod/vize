//! Specialize editor bookkeeping outside the ordinary per-rule hot loop.

use super::{LintVisitor, RuleSelection};
use crate::context::{CallbackSite, LintContext};
use crate::rule::Rule;

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    #[inline]
    pub(super) fn callback_rules(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        selection: RuleSelection<'_>,
        site: CallbackSite,
        visit: impl FnMut(usize, &dyn Rule, &mut LintContext<'a>),
    ) {
        if ctx.callback_order_enabled() {
            ctx.record_callback_site(site);
            Self::callback_rules_inner::<true>(ctx, rules, names, selection, visit);
        } else {
            Self::callback_rules_inner::<false>(ctx, rules, names, selection, visit);
        }
    }

    #[inline]
    fn callback_rules_inner<const ORDERED: bool>(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        selection: RuleSelection<'_>,
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
            RuleSelection::Indices(indices) => {
                for &index in indices {
                    let Some((rule, name)) = rules.get(index).zip(names.get(index).copied()) else {
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
