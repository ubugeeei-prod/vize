//! type/prefer-promise-reject-errors
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/prefer-promise-reject-errors",
    description: "Require Promise rejection reasons to be Error-like values.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/prefer-promise-reject-errors`.
#[derive(Default)]
pub struct PreferPromiseRejectErrors;

impl Rule for PreferPromiseRejectErrors {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
