//! Share an existing semantic Program with tools that also need to print its
//! original source. The two parser profiles have different admission rules.

use super::{EmbedSyntax, Shape, parse_program_for_format};
use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use vize_l0::Allocator;

/// Both views of a script. The caller owns the original semantic observation.
/// The print result is parsed once from its exact bytes and source type.
pub struct ProgramSnapshot<'o, 'a> {
    semantic: &'o EmbedSyntax<'a>,
    printing: Result<Program<'a>, Diagnostics>,
}

impl<'o, 'a> ProgramSnapshot<'o, 'a> {
    /// Prepare the additional printer profile. Any whole-Program observation
    /// can be used, including a semantic refusal. Wrapped grammar shapes
    /// return their actual shape before invoking the printer's parser.
    #[inline]
    pub fn observe_printing(
        arena: &'a Allocator,
        semantic: &'o EmbedSyntax<'a>,
    ) -> Result<Self, Shape> {
        let shape = semantic.grammar().shape;
        if shape != Shape::Program {
            return Err(shape);
        }
        let parsed = parse_program_for_format(
            arena.as_oxc(),
            semantic.source().text(),
            semantic.source_type(),
        );
        let printing = if parsed.diagnostics.is_empty() {
            Ok(parsed.program)
        } else {
            Err(parsed.diagnostics)
        };
        Ok(Self { semantic, printing })
    }

    /// Borrow the caller's original parser observation for analysis passes.
    #[inline]
    #[must_use]
    pub const fn semantic(&self) -> &'o EmbedSyntax<'a> {
        self.semantic
    }

    /// Glyph prints only a clean AST. Preserve every diagnostic on refusal.
    #[inline]
    pub fn for_print(&self) -> Result<&Program<'a>, &Diagnostics> {
        self.printing.as_ref()
    }
}

#[cfg(test)]
mod tests;
