//! type/no-unsafe-enum-comparison
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-enum-comparison",
    description: "Disallow comparing an enum value with a value from a different enum domain.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-enum-comparison`.
#[derive(Default)]
pub struct NoUnsafeEnumComparison;

impl Rule for NoUnsafeEnumComparison {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
