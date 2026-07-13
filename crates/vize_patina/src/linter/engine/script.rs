use vize_carton::ToCompactString;

use crate::LintDiagnostic;
use crate::linter::config::{LintResult, Linter};

impl Linter {
    /// Lint a plain JavaScript/TypeScript module with script-level rules.
    pub fn lint_script(&self, source: &str, filename: &str) -> LintResult {
        let module =
            vize_module::snapshot_module(filename, source, module_language(filename), 0, None);
        self.lint_script_with_shared_artifacts(&module, filename)
    }

    /// Lint a module from the source-faithful Atlas module product.
    ///
    /// The module frontend owns language selection, parse diagnostics, facts,
    /// and control flow. Patina retains its private AST view for AST-only rules
    /// while using the shared module as the production source of identity and
    /// text.
    pub fn lint_script_with_shared_artifacts(
        &self,
        module: &vize_module::ModuleSyntax,
        filename: &str,
    ) -> LintResult {
        let mut result = LintResult {
            filename: filename.to_compact_string(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        };
        let source_end = module
            .base_offset
            .saturating_add(module.source.len() as u32);
        for diagnostic in &module.diagnostics {
            result.diagnostics.push(LintDiagnostic::error(
                "parser/module",
                diagnostic.message.as_ref(),
                diagnostic.span.start.min(source_end),
                diagnostic.span.end.min(source_end),
            ));
            result.error_count += 1;
        }
        super::super::script_rules::append_builtin_script_rules_for_module(
            self,
            module,
            &mut result,
        );
        result
            .diagnostics
            .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        result
    }
}

fn module_language(filename: &str) -> vize_module::ModuleLanguage {
    let clean = filename.split(['?', '#']).next().unwrap_or(filename);
    match clean.rsplit_once('.').map(|(_, extension)| extension) {
        Some("ts" | "mts" | "cts") => vize_module::ModuleLanguage::TypeScript,
        _ => vize_module::ModuleLanguage::JavaScript,
    }
}
