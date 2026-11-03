//! Test-only template entries.

use super::{Allocator, LintResult, Linter, TemplateRuleEnv};
use vize_l0::dialect::VueDialect;

impl Linter {
    /// [`Self::lint_template`] with every rule on the Relief visitor: the
    /// baseline the facade parity tests compare against.
    pub(crate) fn lint_template_relief(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let result = self.lint_template_with_allocator_config(
            &allocator,
            source,
            filename,
            true,
            true,
            TemplateRuleEnv::relief(VueDialect::Vue),
        );
        self.apply_diagnostic_policy(result)
    }

    pub(crate) fn lint_template_rules_only(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator_config(
            &allocator,
            source,
            filename,
            false,
            true,
            TemplateRuleEnv::bare(VueDialect::Vue),
        )
    }
}
