//! type/return-await
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/return-await",
    description: "Enforce consistent use of `return await` based on context and configuration.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/return-await`.
#[derive(Default)]
pub struct ReturnAwait;

impl Rule for ReturnAwait {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
