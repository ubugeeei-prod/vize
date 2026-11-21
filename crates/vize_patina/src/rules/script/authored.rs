//! Run the configured standalone rules on a Program already read by another tool.
use super::{ScriptLintResult, ScriptLinter, SfcScriptContext, script_source_type};
use oxc_span::SourceType;
use vize_l1::embed::authored_program::AuthoredProgram;

/// The supplied parse cannot drive the configured AST-based rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLintRefusal {
    /// The original parser used a different language or module goal.
    DifferentGrammar,
}

impl ScriptLinter {
    /// Parser demand for this rule list. Byte-only rules read the authored block directly.
    #[must_use]
    pub fn authored_source_type(&self) -> Option<SourceType> {
        self.rules
            .iter()
            .any(|rule| rule.uses_ast())
            .then(script_source_type)
    }

    /// Run the configured standalone rules after the other syntax readers.
    /// Rules may annotate OXC semantic IDs, so this final reader consumes the
    /// Program. Every diagnostic offset comes from its original physical block.
    /// A recovered parse still runs byte-only rules in registration order.
    ///
    /// ```compile_fail
    /// use vize_l1::embed::authored_program::AuthoredProgram;
    /// use vize_patina::rules::script::ScriptLinter;
    /// fn reuse(lint: &ScriptLinter, syntax: AuthoredProgram<'_>) {
    ///     let parsed = syntax.parsed();
    ///     let _ = lint.lint_authored(syntax);
    ///     let _ = &parsed.program;
    /// }
    /// ```
    pub fn lint_authored(
        &self,
        syntax: AuthoredProgram<'_>,
    ) -> Result<ScriptLintResult, ScriptLintRefusal> {
        let block = syntax.block();
        if self
            .authored_source_type()
            .is_some_and(|grammar| grammar != syntax.source_type())
        {
            return Err(ScriptLintRefusal::DifferentGrammar);
        }
        let parsed = syntax.parsed();
        let clean = !parsed.panicked && parsed.diagnostics.is_empty();
        let mut result = ScriptLintResult::default();
        for rule in &self.rules {
            if rule.uses_ast() {
                if clean {
                    rule.check_program_with_sfc(
                        &parsed.program,
                        block.source(),
                        block.start() as usize,
                        SfcScriptContext::default(),
                        &mut result,
                    );
                }
            } else {
                rule.check(block.source(), block.start() as usize, &mut result);
            }
        }
        Ok(result)
    }
}
