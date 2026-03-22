//! vapor/require-vapor-attribute
//!
//! Suggest adding the vapor attribute to script setup blocks.
//!
//! This is a informational rule that helps identify components that
//! could be migrated to Vapor mode. Components with `<script setup>`
//! are candidates for Vapor mode optimization.
//!
//! Note: This rule operates at the SFC level, not template level.
//! It's meant to be used with the full SFC source.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vapor/require-vapor-attribute",
    description: "Suggest adding vapor attribute to script setup",
    category: RuleCategory::Vapor,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Suggest vapor attribute on script setup
pub struct RequireVaporAttribute {
    /// Whether to report as suggestion (info) instead of warning
    pub as_suggestion: bool,
}

impl Default for RequireVaporAttribute {
    fn default() -> Self {
        Self {
            as_suggestion: true,
        }
    }
}

impl Rule for RequireVaporAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, _ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // This rule needs access to the full SFC, not just the template.
        // It's a placeholder for SFC-level linting.
        // The actual implementation would check if <script setup> exists
        // without the vapor attribute.
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // This rule requires SFC-level access, not template-level.
        // Tests would be implemented when SFC linting is added.
    }
}
