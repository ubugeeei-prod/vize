//! type/prefer-return-this-type
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/prefer-return-this-type",
    description: "Enforce that `this` is used when only `this` type is returned.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/prefer-return-this-type`.
#[derive(Default)]
pub struct PreferReturnThisType;

impl Rule for PreferReturnThisType {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
