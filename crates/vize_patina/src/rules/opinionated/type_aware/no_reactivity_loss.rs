//! type/no-reactivity-loss
//!
//! Rule definition for the type-aware reactivity loss check.
//!
//! The implementation lives in the native type-aware lint path, where Croquis
//! identifies JavaScript reactivity-flow candidates and Corsa confirms whether
//! the value still has a ref-like wrapper type before reporting.

use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-reactivity-loss",
    description: "Disallow plain snapshots of reactive values across assignments and calls",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata-only rule handle for the native reactivity loss check.
#[derive(Default)]
pub struct NoReactivityLoss;

impl NoReactivityLoss {
    /// Create a new rule handle.
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NoReactivityLoss {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}

#[cfg(test)]
mod tests {
    use super::NoReactivityLoss;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoReactivityLoss::new();
        assert_eq!(rule.meta().name, "type/no-reactivity-loss");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }
}
