//! type/no-redundant-type-constituents
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-redundant-type-constituents",
    description: "Disallow members of unions and intersections that do nothing or override type information.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-redundant-type-constituents`.
#[derive(Default)]
pub struct NoRedundantTypeConstituents;

impl Rule for NoRedundantTypeConstituents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
