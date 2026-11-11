//! type/no-unsafe-member-access
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-member-access",
    description: "Disallow member access on values typed as any.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/no-unsafe-member-access`.
#[derive(Default)]
pub struct NoUnsafeMemberAccess;

impl Rule for NoUnsafeMemberAccess {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
