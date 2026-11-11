//! type/use-unknown-in-catch-callback-variable
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/use-unknown-in-catch-callback-variable",
    description: "Require Promise catch callback variables to use an explicit unknown annotation.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/use-unknown-in-catch-callback-variable`.
#[derive(Default)]
pub struct UseUnknownInCatchCallbackVariable;

impl Rule for UseUnknownInCatchCallbackVariable {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
