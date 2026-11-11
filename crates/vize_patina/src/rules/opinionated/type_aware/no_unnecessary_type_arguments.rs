//! type/no-unnecessary-type-arguments
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unnecessary-type-arguments",
    description: "Disallow type arguments that are equal to the default.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unnecessary-type-arguments`.
#[derive(Default)]
pub struct NoUnnecessaryTypeArguments;

impl Rule for NoUnnecessaryTypeArguments {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
