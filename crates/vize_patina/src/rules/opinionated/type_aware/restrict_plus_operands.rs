//! type/restrict-plus-operands
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/restrict-plus-operands",
    description: "Require plus operands to be explicitly compatible.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/restrict-plus-operands`.
#[derive(Default)]
pub struct RestrictPlusOperands;

impl Rule for RestrictPlusOperands {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
