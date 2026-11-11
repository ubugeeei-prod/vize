//! type/no-duplicate-type-constituents
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-duplicate-type-constituents",
    description: "Disallow duplicate union or intersection constituents.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-duplicate-type-constituents`.
#[derive(Default)]
pub struct NoDuplicateTypeConstituents;

impl Rule for NoDuplicateTypeConstituents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
