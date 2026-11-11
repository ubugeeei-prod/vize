//! type/prefer-reduce-type-parameter
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/prefer-reduce-type-parameter",
    description: "Prefer Array#reduce type parameters over default-value assertions.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/prefer-reduce-type-parameter`.
#[derive(Default)]
pub struct PreferReduceTypeParameter;

impl Rule for PreferReduceTypeParameter {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
