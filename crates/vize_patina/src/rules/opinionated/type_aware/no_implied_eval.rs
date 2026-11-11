//! type/no-implied-eval
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-implied-eval",
    description: "Disallow string-based dynamic code execution APIs.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-implied-eval`.
#[derive(Default)]
pub struct NoImpliedEval;

impl Rule for NoImpliedEval {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
