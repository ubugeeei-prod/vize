//! Opt-in metadata; native Corsa evaluates the actual condition types.
use crate::{
    diagnostic::Severity,
    rule::{Rule, RuleCategory, RuleMeta},
};

static META: RuleMeta = RuleMeta {
    name: "type/strict-boolean-expressions",
    description: "Require safe boolean expressions in script and template conditions",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct StrictBooleanExpressions;

impl Rule for StrictBooleanExpressions {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
