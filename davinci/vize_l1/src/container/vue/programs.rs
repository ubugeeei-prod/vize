//! Original Program syntax shared by consumers of one Vue descriptor.

use super::{AdmittedDescriptor, ScriptRole, ScriptView};
use crate::embed::{
    EmbedSource,
    syntax::{EmbedSyntax, ProgramOptions, parse_program_once},
};
use vize_l0::Allocator;

/// The actual authored Programs for both selected roles in one Vue SFC.
/// Each present role, including a recovered script, retains its own syntax.
/// For a single role use `ScriptView::observe_program` on that selection.
#[derive(Clone, Copy)]
pub struct ScriptPrograms<'p, 'o, 'a> {
    descriptor: &'p AdmittedDescriptor<'o, 'a>,
    ordinary: Option<&'p EmbedSyntax<'a>>,
    setup: Option<&'p EmbedSyntax<'a>>,
}

impl<'a> ScriptView<'_, 'a> {
    /// Observe this original script once. The descriptor supplies the root,
    /// selected UTF-8 content, language and Module goal.
    #[inline]
    #[must_use]
    pub fn observe_program(self, arena: &'a Allocator) -> EmbedSyntax<'a> {
        parse_program_once(
            arena,
            EmbedSource::from_block(self.block()),
            ProgramOptions::module(self.lang()),
        )
    }
}

impl<'o, 'a> AdmittedDescriptor<'o, 'a> {
    /// Run consumers together on the Programs of every selected script. The
    /// observations stay in place for the duration of the callback. Owned
    /// results, such as an L2 File or formatted text, may leave the callback.
    #[inline]
    pub fn with_script_programs<R>(
        self,
        arena: &'a Allocator,
        read: impl FnOnce(ScriptPrograms<'_, 'o, 'a>) -> R,
    ) -> R {
        let ordinary = self.ordinary().map(|s| s.observe_program(arena));
        let setup = self.setup().map(|s| s.observe_program(arena));
        read(ScriptPrograms {
            descriptor: &self,
            ordinary: ordinary.as_ref(),
            setup: setup.as_ref(),
        })
    }
}

impl<'p, 'o, 'a> ScriptPrograms<'p, 'o, 'a> {
    /// The original descriptor that selected the roles, spans and languages.
    #[inline]
    #[must_use]
    pub fn descriptor(&self) -> AdmittedDescriptor<'o, 'a> {
        *self.descriptor
    }

    /// Borrow the original semantic observation for a role. An absent role
    /// yields `None`. A selected malformed script keeps its full observation
    /// with the parser's diagnostics and typed hole.
    #[inline]
    #[must_use]
    pub fn get(&self, role: ScriptRole) -> Option<&'p EmbedSyntax<'a>> {
        match role {
            ScriptRole::Ordinary => self.ordinary,
            ScriptRole::Setup => self.setup,
        }
    }
}

#[cfg(test)]
mod tests;
