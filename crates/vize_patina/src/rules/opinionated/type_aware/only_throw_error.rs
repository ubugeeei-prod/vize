//! type/only-throw-error
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/only-throw-error",
    description: "Require thrown values to be Error-like.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/only-throw-error`.
#[derive(Default)]
pub struct OnlyThrowError;

impl Rule for OnlyThrowError {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
