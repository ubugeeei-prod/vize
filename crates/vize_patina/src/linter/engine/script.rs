use vize_l0::ToCompactString;

use crate::diagnostic::Severity;
use crate::linter::config::{LintResult, Linter};

impl Linter {
    /// Lint a plain JavaScript/TypeScript module with script-level rules.
    pub fn lint_script(&self, source: &str, filename: &str) -> LintResult {
        let mut result = LintResult {
            filename: filename.to_compact_string(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        };

        super::super::script_rules::append_builtin_script_rules_for_source(
            self,
            source,
            0,
            &mut result,
        );
        #[cfg(not(target_arch = "wasm32"))]
        super::super::native_type_aware::plain_module::append_plain_module_type_diagnostics(
            self,
            source,
            filename,
            &mut result,
        );
        self.finish_script_result(source, result)
    }

    pub(super) fn finish_script_result(&self, source: &str, mut result: LintResult) -> LintResult {
        crate::context::retain_unless_source_suppressed(source, &mut result.diagnostics);
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count();
        result.warning_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count();
        result
            .diagnostics
            .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        self.apply_diagnostic_policy(result)
    }
}
