//! An original whole-file TypeScript declaration parsed with the declaration grammar.
use super::authored_program::AuthoredProgram;
use oxc_ast::ast::Program;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};

/// Keeps the input and both recovery views tied to the same parse.
pub struct DeclarationProgram<'a>(AuthoredProgram<'a>);

impl<'a> DeclarationProgram<'a> {
    #[must_use]
    pub fn parse(arena: &'a Allocator, root: SourceRoot<'a>) -> Self {
        Self(AuthoredProgram::parse(
            arena,
            root.whole_block(),
            SourceType::d_ts(),
        ))
    }

    #[must_use]
    pub fn source(&self) -> &'a str {
        self.0.block().source()
    }

    /// Some consumers can inspect a recovered tree with diagnostics.
    #[must_use]
    pub fn recovered(&self) -> Option<&Program<'a>> {
        let parsed = self.0.parsed();
        (!parsed.panicked).then_some(&parsed.program)
    }

    /// Projection and completion require a successful parse without diagnostics.
    #[must_use]
    pub fn clean(&self) -> Option<&Program<'a>> {
        if self.0.parsed().diagnostics.is_empty() {
            self.recovered()
        } else {
            None
        }
    }
}
