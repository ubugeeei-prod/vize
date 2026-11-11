//! type/restrict-template-expressions
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/restrict-template-expressions",
    description: "Require template literal expressions to be string-compatible.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/restrict-template-expressions`.
#[derive(Default)]
pub struct RestrictTemplateExpressions;

impl Rule for RestrictTemplateExpressions {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
