//! Immutable authority from the original parser's complete observation.

use oxc_ast::ast::{Comment, Program};
use oxc_diagnostics::Diagnostics;
use oxc_span::SourceType;

use crate::{
    ParseOptions, Parser, ParserReturn,
    config::{NoTokensParserConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig},
};

mod default_export;
pub use default_export::OriginalDefaultExport;

/// The complete ordinarily owned result of one parser invocation.
///
/// Private storage prevents a caller from discarding diagnostics and presenting
/// a recovered AST as a clean parse. This is syntax admission, not semantic
/// validation. There is no mutable AST access or unchecked extraction.
/// Shared structure does not freeze the AST's semantic ID Cells.
///
/// Raw public parser returns cannot establish this authority:
///
/// ```compile_fail
/// use oxc_parser::{ParserReturn, ProgramObservation};
/// fn forge<'a>(parsed: ParserReturn<'a>) -> ProgramObservation<'a> {
///     ProgramObservation { parsed, source_text: "", source_type: Default::default(), options: Default::default() }
/// }
/// ```
pub struct ProgramObservation<'a> {
    parsed: ParserReturn<'a>,
    source_text: &'a str,
    source_type: SourceType,
    options: ParseOptions,
}

/// A short immutable borrow minted only by a clean original observation.
///
/// The owner borrow cannot be replaced with an arbitrary Program:
///
/// ```compile_fail
/// use oxc_parser::{AdmittedProgram, ProgramObservation};
/// fn forge<'p, 'a>(observation: &'p ProgramObservation<'a>) -> AdmittedProgram<'p, 'a> {
///     AdmittedProgram { observation }
/// }
/// ```
///
/// A live capability exposes only shared syntax:
///
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use oxc_parser::AdmittedProgram;
/// fn mutate<'p, 'a>(admitted: &'p AdmittedProgram<'p, 'a>) -> &'p mut Program<'a> {
///     admitted.program()
/// }
/// ```
pub struct AdmittedProgram<'p, 'a> {
    observation: &'p ProgramObservation<'a>,
}

impl<'a> Parser<'a, NoTokensParserConfig> {
    /// Retain the actual source/profile/options and complete result of one parse.
    ///
    /// Only the parser's built-in configurations can establish this authority.
    /// A custom configuration still supports ordinary `parse`, but cannot
    /// substitute lexer handlers and mint native compiler admission:
    ///
    /// ```compile_fail,E0599
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::{Parser, config::{NoTokensLexerConfig, ParserConfig}};
    /// use oxc_span::SourceType;
    /// #[derive(Default)]
    /// struct Custom;
    /// impl ParserConfig for Custom {
    ///     type LexerConfig = NoTokensLexerConfig;
    ///     fn lexer_config(&self) -> NoTokensLexerConfig { NoTokensLexerConfig }
    /// }
    /// let allocator = Allocator::default();
    /// Parser::new(&allocator, "const value = 1;", SourceType::mjs())
    ///     .with_config(Custom).parse_observed();
    /// ```
    pub fn parse_observed(self) -> ProgramObservation<'a> {
        observe_parser(self)
    }
}

impl<'a> Parser<'a, TokensParserConfig> {
    /// Retain one complete observation using the built-in token configuration.
    pub fn parse_observed(self) -> ProgramObservation<'a> {
        observe_parser(self)
    }
}

impl<'a> Parser<'a, RuntimeParserConfig> {
    /// Retain one complete observation using the built-in runtime configuration.
    pub fn parse_observed(self) -> ProgramObservation<'a> {
        observe_parser(self)
    }
}

// Private sharing cannot grant an unknown ParserConfig a public authority path.
fn observe_parser<'a, C: ParserConfig>(parser: Parser<'a, C>) -> ProgramObservation<'a> {
    let source_text = parser.source_text;
    let source_type = parser.source_type;
    let options = parser.options;
    let parsed = parser.parse();
    ProgramObservation { parsed, source_text, source_type, options }
}

impl<'a> ProgramObservation<'a> {
    /// Recovered, fatal and Flow results never mint this capability.
    #[must_use]
    pub fn admitted(&self) -> Option<AdmittedProgram<'_, 'a>> {
        (!self.parsed.panicked
            && !self.parsed.is_flow_language
            && !self.parsed.diagnostics.has_errors())
        .then_some(AdmittedProgram { observation: self })
    }

    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.parsed.diagnostics
    }

    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        self.parsed.program.comments.as_slice()
    }

    #[must_use]
    pub fn panicked(&self) -> bool {
        self.parsed.panicked
    }

    #[must_use]
    pub fn is_flow_language(&self) -> bool {
        self.parsed.is_flow_language
    }
}

impl<'p, 'a> AdmittedProgram<'p, 'a> {
    #[must_use]
    pub fn program(&self) -> &'p Program<'a> {
        &self.observation.parsed.program
    }

    /// Actual original bytes, including comments and line endings.
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.observation.source_text
    }

    /// Whether the original lexer decoded a legacy numeric literal or string
    /// escape forbidden in a strict module. This does not certify semantics.
    /// Syntax diagnostics and ordinary parser output remain unchanged.
    #[must_use]
    pub fn has_legacy_literals(&self) -> bool {
        self.observation.parsed.has_legacy_literals
    }

    /// Original requested profile, before any Unambiguous inference.
    #[must_use]
    pub fn source_type(&self) -> SourceType {
        self.observation.source_type
    }

    /// Actual parser options; admission does not imply default compiler options.
    #[must_use]
    pub fn options(&self) -> ParseOptions {
        self.observation.options
    }
}

#[cfg(test)]
mod legacy_literal_tests {
    use super::*;
    use oxc_allocator::Allocator;

    #[test]
    fn completed_tail_receipt_survives_committed_unambiguous_await_reparse() {
        let arena = Allocator::default();
        for literal in ["010", "'\\8'"] {
            let source = format!("await /x/u; export {{}}; const value={literal};");
            let parsed = Parser::new(&arena, &source, SourceType::unambiguous()).parse_observed();
            let admitted = parsed.admitted().expect("completed original syntax");
            assert!(admitted.has_legacy_literals(), "{literal}");
            assert!(admitted.program().source_type.is_module());
        }
    }

    #[test]
    fn original_decoder_retains_legacy_facts_without_changing_syntax_admission() {
        let arena = Allocator::default();
        for literal in ["010", "08", "09.5", "'\\1'", "'\\8'", "'\\9'", "'\\00'"] {
            let source = format!("const value={literal};");
            let parsed = Parser::new(&arena, &source, SourceType::mjs()).parse_observed();
            let admitted = parsed.admitted().expect("unchanged syntax admission");
            assert!(admitted.has_legacy_literals(), "{literal}");
            assert!(parsed.diagnostics().is_empty());
            assert_eq!(admitted.source(), source);
        }
        for literal in [
            "0",
            "0o10",
            "0x10",
            "0b10",
            "0.1",
            "0e1",
            "'雪'",
            "'\\0'",
            "'\\x01'",
            "'\\u0001'",
            "'\\\\1'",
            "'\\🌸'",
        ] {
            let source = format!("const value={literal};");
            let parsed = Parser::new(&arena, &source, SourceType::mjs()).parse_observed();
            assert!(
                !parsed.admitted().expect("actual admission").has_legacy_literals(),
                "{literal}"
            );
        }
    }
}
