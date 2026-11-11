//! type/related-getter-setter-pairs
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/related-getter-setter-pairs",
    description: "Require paired getters and setters to use compatible types.",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata for `type/related-getter-setter-pairs`.
#[derive(Default)]
pub struct RelatedGetterSetterPairs;

impl Rule for RelatedGetterSetterPairs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
