//! An option-bearing instance preserves the public unit struct construction.
use super::{DirectiveNode, ElementNode, LintContext, ValidVSlot};
use crate::rule::{Rule, RuleMeta};

struct ConfiguredValidVSlot {
    allow_modifiers: bool,
}

impl ValidVSlot {
    pub(crate) fn configured(allow_modifiers: bool) -> impl Rule {
        ConfiguredValidVSlot { allow_modifiers }
    }
}

impl Rule for ConfiguredValidVSlot {
    fn meta(&self) -> &'static RuleMeta {
        ValidVSlot.meta()
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        ValidVSlot.enter_element(ctx, element);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        ValidVSlot.check_slot_directive(ctx, element, directive, self.allow_modifiers);
    }
}
