//! Configured tool products on the same immutable editor revision.
use super::{ScriptQueryProducts, ScriptQueryRefusal, ServerState, SourceSnapshot, script_frame};
use vize_canon::{ScriptDiagnosticProfile, SfcTypeDiagnostic};
use vize_glyph::{FormatError, FormatResult, GlyphFormatter};
use vize_l0::Span;
use vize_l1::embed::authored_program::visit_authored_profiles;
use vize_patina::rules::script::{ScriptLintResult, ScriptLinter};

/// The configured standalone rules' result for one physical script body.
#[derive(Debug)]
pub struct ScriptBlockLint {
    pub span: Span,
    pub result: ScriptLintResult,
}

/// Script products in ordinary-script then setup-script order.
#[derive(Debug)]
pub struct ScriptToolAnalysis {
    pub editor: ScriptQueryProducts,
    pub syntax_diagnostics: Vec<SfcTypeDiagnostic>,
    pub script_lint: Option<ScriptBlockLint>,
    pub setup_lint: Option<ScriptBlockLint>,
}

/// The full SFC formatter and the script readers can recover independently.
#[derive(Debug)]
pub struct ScriptToolProducts {
    pub format: Result<FormatResult, FormatError>,
    /// Absent when the editor's original descriptor rejects the document.
    pub scripts: Option<ScriptToolAnalysis>,
}

impl SourceSnapshot {
    /// Publish the selected Glyph formatting, configured Patina rules, Canon script
    /// syntax diagnostics and Maestro outline and module specifiers for one revision.
    pub fn script_tool_products(
        &self,
        state: &ServerState,
        linter: &ScriptLinter,
        formatter: &GlyphFormatter<'_>,
    ) -> Result<ScriptToolProducts, ScriptQueryRefusal> {
        let mut script_lint = None;
        let mut setup_lint = None;
        let mut syntax_diagnostics = Vec::new();
        let editor = self.read_script_blocks(state, |out, root, role, block| {
            let (frame, prefix) = script_frame(root, block)?;
            let lint = if block.setup {
                &mut setup_lint
            } else {
                &mut script_lint
            };
            let lang = block.lang.as_deref();
            let rules = linter.authored_rules();
            let lint_type = rules.source_type();
            let editor_type = ScriptQueryProducts::script_type(lang);
            let original_checker = ScriptDiagnosticProfile::for_language(lang);
            let checker = original_checker.sharing_plain_typescript(lint_type);
            let checker_type = checker.source_type();
            // These editor reads inspect syntax, positions and module specifiers.
            // The final lint rules retain their actual filename-bearing Program.
            let editor_type = if editor_type == original_checker.source_type() {
                checker_type
            } else {
                editor_type
            };
            visit_authored_profiles(
                frame,
                &[Some(editor_type), Some(checker_type), lint_type],
                |syntax| {
                    let ty = syntax.source_type();
                    if ty == editor_type {
                        out.read_tool_editor_products(role, block, prefix, &syntax);
                    }
                    if ty == checker_type
                        && (syntax.parsed().panicked || !syntax.parsed().diagnostics.is_empty())
                    {
                        syntax_diagnostics.extend(
                            checker
                                .diagnostics(&syntax)
                                .map_err(ScriptQueryRefusal::Checker)?,
                        );
                    }
                    if Some(ty) == lint_type {
                        // Configured semantic rules can annotate the AST. Give them the last read.
                        let result = rules
                            .lint_authored(syntax)
                            .map_err(ScriptQueryRefusal::Linter)?;
                        *lint = Some(ScriptBlockLint {
                            span: frame.span(),
                            result,
                        });
                    }
                    Ok(())
                },
            )?;
            if lint_type.is_none() {
                let result = rules.lint_source(frame);
                *lint = Some(ScriptBlockLint {
                    span: frame.span(),
                    result,
                });
            }
            Ok(())
        })?;
        let source = self.source();
        let template_only = if editor.is_some() && script_lint.is_none() && setup_lint.is_none() {
            vize_l1::container::Vue.observe_template_only(source)
        } else {
            None
        };
        let format = match template_only {
            Some(template) => formatter.format_observed_template(&template),
            None => formatter.format(source),
        };
        self.check_current(&state.documents)
            .map_err(ScriptQueryRefusal::Host)?;
        let scripts = editor.map(|editor| ScriptToolAnalysis {
            editor,
            syntax_diagnostics,
            script_lint,
            setup_lint,
        });
        Ok(ScriptToolProducts { format, scripts })
    }
}

#[cfg(test)]
mod tests;
