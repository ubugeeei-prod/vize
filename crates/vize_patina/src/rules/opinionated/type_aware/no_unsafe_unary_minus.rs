//! type/no-unsafe-unary-minus
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-unary-minus",
    description: "Disallow unary negation on non-number and non-bigint values.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-unary-minus`.
#[derive(Default)]
pub struct NoUnsafeUnaryMinus;

impl Rule for NoUnsafeUnaryMinus {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
