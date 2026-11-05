//! Specialize editor bookkeeping outside the ordinary per-rule hot loop.

use super::LintVisitor;
use crate::context::{CallbackSite, LintContext};
use crate::rule::Rule;

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    #[inline]
    pub(super) fn callback_rules<const INCLUDE_MASKED: bool>(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        keep: Option<&[bool]>,
        site: CallbackSite,
        visit: impl FnMut(usize, &dyn Rule, &mut LintContext<'a>),
    ) {
        if ctx.callback_order_enabled() {
            ctx.record_callback_site(site);
            Self::callback_rules_inner::<true, INCLUDE_MASKED>(ctx, rules, names, keep, visit);
        } else {
            Self::callback_rules_inner::<false, INCLUDE_MASKED>(ctx, rules, names, keep, visit);
        }
    }

    #[inline]
    fn callback_rules_inner<const ORDERED: bool, const INCLUDE_MASKED: bool>(
        ctx: &mut LintContext<'a>,
        rules: &[Box<dyn Rule>],
        names: &[&'static str],
        keep: Option<&[bool]>,
        mut visit: impl FnMut(usize, &dyn Rule, &mut LintContext<'a>),
    ) {
        for (index, (rule, name)) in rules.iter().zip(names.iter().copied()).enumerate() {
            if !INCLUDE_MASKED && !Self::rule_active(keep, index) {
                continue;
            }
            if ORDERED {
                ctx.set_callback_rule(index);
            }
            ctx.current_rule = name;
            visit(index, rule.as_ref(), ctx);
        }
    }
}
