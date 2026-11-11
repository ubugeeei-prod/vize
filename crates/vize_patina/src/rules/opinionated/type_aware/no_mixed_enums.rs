//! type/no-mixed-enums
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-mixed-enums",
    description: "Disallow mixing string and numeric enum members.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-mixed-enums`.
#[derive(Default)]
pub struct NoMixedEnums;

impl Rule for NoMixedEnums {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
