//! type/unbound-method
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/unbound-method",
    description: "Disallow referencing methods without their receiver (`this`).",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/unbound-method`.
#[derive(Default)]
pub struct UnboundMethod;

impl Rule for UnboundMethod {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
