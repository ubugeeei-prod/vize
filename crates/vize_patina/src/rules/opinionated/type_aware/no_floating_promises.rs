//! type/no-floating-promises
//!
//! Rule definition for the type-aware floating Promise check.
//!
//! The actual detection logic lives in the native `corsa`-backed lint path:
//! `crate::linter::native_type_aware`.
//! This rule object exists so the rule can be registered, enabled, and exposed
//! through presets without duplicating Promise heuristics in the rule layer.

use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-floating-promises",
    description: "Disallow floating (unhandled) Promises",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata-only rule handle for the native floating Promise check.
#[derive(Default)]
pub struct NoFloatingPromises;

impl NoFloatingPromises {
    /// Create a new rule handle.
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NoFloatingPromises {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}

#[cfg(test)]
mod tests {
    use super::NoFloatingPromises;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoFloatingPromises::new();
        assert_eq!(rule.meta().name, "type/no-floating-promises");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }
}
