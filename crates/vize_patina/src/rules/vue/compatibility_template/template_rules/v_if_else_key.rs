//! Keys on repeated conditional components.

use super::shared::check_conditional_siblings;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/v-if-else-key",
    description: "Require keys on repeated conditional components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/v-if-else-key`.
pub struct VIfElseKey;

impl Rule for VIfElseKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_conditional_siblings(ctx, &element.children);
    }
}
