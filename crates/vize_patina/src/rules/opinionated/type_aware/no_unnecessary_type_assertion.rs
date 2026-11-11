//! type/no-unnecessary-type-assertion
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-type-assertion",
    description: "Disallow type assertions that do not change the expression type.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-type-assertion`.
#[derive(Default)]
pub struct NoUnnecessaryTypeAssertion;

impl Rule for NoUnnecessaryTypeAssertion {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
