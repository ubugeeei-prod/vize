//! type/no-unnecessary-type-conversion
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-type-conversion",
    description: "Disallow conversions that do not change the type or value of the expression.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-type-conversion`.
#[derive(Default)]
pub struct NoUnnecessaryTypeConversion;

impl Rule for NoUnnecessaryTypeConversion {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
