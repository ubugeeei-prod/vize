//! Standalone HTML execution with an optional Levels petite-vue view.

use super::{Allocator, LintResult, Linter, TemplateRuleEnv};
use vize_l0::dialect::standalone_html_dialect;

impl Linter {
    /// Lint a standalone HTML document that may use Vue from a CDN.
    #[inline]
    pub fn lint_standalone_html(&self, source: &str, filename: &str) -> LintResult {
        self.lint_standalone_html_with_level(source, filename, true)
    }

    #[cfg(test)]
    pub(crate) fn lint_standalone_html_relief_for_test(
        &self,
        source: &str,
        filename: &str,
    ) -> LintResult {
        self.lint_standalone_html_with_level(source, filename, false)
    }

    fn lint_standalone_html_with_level(
        &self,
        source: &str,
        filename: &str,
        use_level: bool,
    ) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        // Resolve the document dialect so dialect-specific rules (e.g.
        // require-v-for-key, which petite-vue does not require) can gate
        // themselves on petite-vue documents.
        let dialect = standalone_html_dialect(None, source);
        let petite_document_demand = ["petite-vue/valid-v-effect", "petite-vue/valid-v-scope"]
            .into_iter()
            .any(|name| self.registry.has_rule(name) && self.is_rule_enabled(name));
        let native_document = (use_level && dialect.is_petite_vue() && petite_document_demand)
            .then(|| vize_l0::SourceRoot::new(source).ok())
            .flatten()
            .map(|root| vize_l1::markup::document::NativeDocument::lex_in(&allocator, root));
        let petite_document = native_document.as_ref().and_then(|owner| {
            vize_l2::dialect::petite::PetiteDocument::lower_in(&allocator, owner).ok()
        });
        let mut env = TemplateRuleEnv::relief(dialect);
        env.petite_document = petite_document.as_ref();
        let mut result = self
            .lint_template_with_allocator_config(&allocator, source, filename, false, false, env);

        if super::super::script_rules::has_active_builtin_script_rules(self) {
            super::super::script_rules::append_builtin_script_diagnostics_from_html(
                self,
                source,
                &mut result,
            );
            result
                .diagnostics
                .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        }

        self.apply_diagnostic_policy(result)
    }
}
