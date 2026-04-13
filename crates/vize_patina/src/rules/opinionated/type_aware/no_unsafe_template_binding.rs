//! type/no-unsafe-template-binding
//!
//! Rule definition for Corsa-backed template expression checks.
//!
//! The implementation lives in `crate::linter::native_type_aware`, where Vue
//! template AST locations are resolved against Corsa type queries. This rule
//! handle only carries metadata so presets can opt into the native check.

use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "type/no-unsafe-template-binding",
    description: "Disallow template bindings that resolve to unsafe types",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Metadata-only rule handle for Corsa-backed template binding checks.
#[derive(Default)]
pub struct NoUnsafeTemplateBinding;

impl NoUnsafeTemplateBinding {
    /// Create a new rule handle.
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NoUnsafeTemplateBinding {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnsafeTemplateBinding;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoUnsafeTemplateBinding::new();
        assert_eq!(rule.meta().name, "type/no-unsafe-template-binding");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }
}
