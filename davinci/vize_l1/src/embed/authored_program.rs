//! Original whole-script syntax for tools using the stock parser options.
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock};

/// A syntax reader may inspect a recovered tree and its complete diagnostics.
/// The parse profile and byte frame stay attached to the original parser result.
pub struct AuthoredProgram<'a> {
    block: SourceBlock<'a>,
    source_type: SourceType,
    parsed: ParserReturn<'a>,
}

impl<'a> AuthoredProgram<'a> {
    #[must_use]
    pub fn parse(arena: &'a Allocator, block: SourceBlock<'a>, source_type: SourceType) -> Self {
        let parsed = Parser::new(arena.as_oxc(), block.source(), source_type).parse();
        Self {
            block,
            source_type,
            parsed,
        }
    }

    #[must_use]
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    /// The requested profile survives the parser's resolution of Unambiguous inputs.
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[must_use]
    pub const fn parsed(&self) -> &ParserReturn<'a> {
        &self.parsed
    }
}
