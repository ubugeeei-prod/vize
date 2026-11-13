//! type/no-unnecessary-type-parameters
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-type-parameters",
    description: "Disallow type parameters that are used only once in their signature.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-type-parameters`.
#[derive(Default)]
pub struct NoUnnecessaryTypeParameters;

impl Rule for NoUnnecessaryTypeParameters {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
