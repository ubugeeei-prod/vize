//! Run the configured standalone rules on a Program already read by another tool.
use super::{ScriptLintResult, ScriptLinter, ScriptRule, SfcScriptContext};
use oxc_ast::ast::Program;
use oxc_span::{FileExtension, SourceType};
use vize_l0::SourceBlock;
use vize_l1::embed::authored_program::AuthoredProgram;

/// The supplied parse cannot drive the configured AST-based rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLintRefusal {
    /// The original parser used a different language or module goal.
    DifferentGrammar,
}

/// The parser demand of one block's configured standalone rule pass.
pub struct AuthoredScriptRules<'l> {
    linter: &'l ScriptLinter,
    grammar: Option<SourceType>,
}

impl ScriptLinter {
    /// Inspect parser demand once before scheduling the readers of this block.
    #[must_use]
    pub fn authored_rules(&self) -> AuthoredScriptRules<'_> {
        AuthoredScriptRules {
            linter: self,
            grammar: self.authored_source_type(),
        }
    }

    /// Parser demand for this rule list. Byte-only rules read the authored block directly.
    #[must_use]
    pub fn authored_source_type(&self) -> Option<SourceType> {
        self.rules
            .iter()
            .any(|rule| rule.uses_ast())
            .then(|| SourceType::from(FileExtension::Ts))
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
        self.authored_rules().lint_authored(syntax)
    }
}

impl AuthoredScriptRules<'_> {
    /// The complete parser profile required for this block, when any rule needs syntax.
    #[must_use]
    pub const fn source_type(&self) -> Option<SourceType> {
        self.grammar
    }

    /// Return the complete empty result when every configured rule consumes syntax
    /// and its required spelling is absent in this physical script. Byte readers,
    /// undeclared spellings and escaped identifiers keep their original evaluation.
    pub fn empty_result_for(&self, block: SourceBlock<'_>) -> Option<ScriptLintResult> {
        let bytes = block.source().as_bytes();
        let mut escaped = None;
        self.linter
            .rules
            .iter()
            .all(|rule| rule.uses_ast() && !may_match(rule.as_ref(), bytes, &mut escaped))
            .then(ScriptLintResult::default)
    }

    /// A caller without shared syntax can request the complete standalone result.
    pub fn lint_source(&self, block: SourceBlock<'_>) -> ScriptLintResult {
        if self.grammar.is_some() {
            return self.linter.lint(block.source(), block.start() as usize);
        }
        self.run(block, None)
    }

    /// Consume the original parse after read-only clients have finished.
    pub fn lint_authored(
        &self,
        syntax: AuthoredProgram<'_>,
    ) -> Result<ScriptLintResult, ScriptLintRefusal> {
        if self
            .grammar
            .is_some_and(|grammar| grammar != syntax.source_type())
        {
            return Err(ScriptLintRefusal::DifferentGrammar);
        }
        let parsed = syntax.parsed();
        let clean = !parsed.panicked && parsed.diagnostics.is_empty();
        Ok(self.run(syntax.block(), clean.then_some(&parsed.program)))
    }

    fn run<'p>(
        &self,
        block: SourceBlock<'_>,
        program: Option<&'p Program<'p>>,
    ) -> ScriptLintResult {
        let mut result = ScriptLintResult::default();
        let bytes = block.source().as_bytes();
        let mut escaped = None;
        for rule in &self.linter.rules {
            if rule.uses_ast() {
                if let Some(program) = program {
                    if !may_match(rule.as_ref(), bytes, &mut escaped) {
                        continue;
                    }
                    rule.check_program_with_sfc(
                        program,
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
        result
    }
}

fn may_match(rule: &dyn ScriptRule, bytes: &[u8], escaped: &mut Option<bool>) -> bool {
    rule.required_authored_spelling().is_none_or(|spelling| {
        memchr::memmem::find(bytes, spelling.as_bytes()).is_some()
            || *escaped.get_or_insert_with(|| memchr::memchr(b'\\', bytes).is_some())
    })
}
