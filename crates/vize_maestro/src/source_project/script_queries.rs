//! Script syntax queries over the current editor's immutable source and descriptor.
#![expect(
    clippy::disallowed_types,
    reason = "move the original file-rename visitor's owned string without reallocating it"
)]

use super::{SnapshotRefusal, SourceSnapshot};
use crate::server::ServerState;
use oxc_span::SourceType;
use tower_lsp::lsp_types::DocumentSymbol;
use vize_atelier_sfc::{SfcDescriptor, SfcScriptBlock};
use vize_incremental::SharedDescriptor;
use vize_l0::{Allocator, SourceBlock, SourceFrameError, SourceRoot, Span, line_index::LineIndex};
use vize_l1::embed::authored_program::AuthoredProgram;

/// A literal module specifier selected by the editor's file-rename reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptModuleSpecifier {
    pub span: Span,
    pub specifier: std::string::String,
}

/// The two independent editor reads in their original per-role order.
#[derive(Debug, Default, PartialEq)]
pub struct ScriptQueryProducts {
    pub symbols: Vec<DocumentSymbol>,
    pub module_specifiers: Vec<ScriptModuleSpecifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptQueryRefusal {
    Host(SnapshotRefusal),
    Language,
    Frame(SourceFrameError),
    DescriptorSourceMismatch,
    #[cfg(feature = "experimental-checker-script-queries")]
    Checker(vize_canon::ScriptDiagnosticRefusal),
    #[cfg(feature = "experimental-script-tools")]
    Linter(vize_patina::rules::script::ScriptLintRefusal),
}

impl SourceSnapshot {
    /// Read the script outline and the file-rename module specifiers together.
    /// Empty blocks contribute their outline directly. Each nonempty block is
    /// parsed once with its original JS, JSX, TS or TSX grammar. Concurrent edits
    /// can proceed during the read and invalidate publication.
    pub fn script_query_products(
        &self,
        state: &ServerState,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        self.with_script_blocks(state, |products, root, name, block| {
            products.read_script(root, name, block, None, |_| Ok(()))
        })
    }

    fn with_script_blocks(
        &self,
        state: &ServerState,
        read: impl FnMut(
            &mut ScriptQueryProducts,
            SourceRoot<'_>,
            &'static str,
            &SfcScriptBlock<'_>,
        ) -> Result<(), ScriptQueryRefusal>,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        let result = self.read_script_blocks(state, read)?;
        self.check_current(&state.documents)
            .map_err(ScriptQueryRefusal::Host)?;
        Ok(result)
    }

    fn with_current_script_descriptor<T>(
        &self,
        state: &ServerState,
        read: impl FnOnce(Option<SharedDescriptor>) -> Result<T, ScriptQueryRefusal>,
    ) -> Result<T, ScriptQueryRefusal> {
        self.with_current(&state.documents, |_| {
            if self.language_id() != "vue" || crate::utils::is_jsx_path(self.uri().path()) {
                return Err(ScriptQueryRefusal::Language);
            }
            read(state.sfc_descriptor(self.uri(), self.source()))
        })
        .map_err(ScriptQueryRefusal::Host)?
    }

    fn read_script_blocks(
        &self,
        state: &ServerState,
        read: impl FnMut(
            &mut ScriptQueryProducts,
            SourceRoot<'_>,
            &'static str,
            &SfcScriptBlock<'_>,
        ) -> Result<(), ScriptQueryRefusal>,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        let parsed = self.with_current_script_descriptor(state, Ok)?;
        self.read_descriptor_blocks(parsed.as_deref(), read)
    }

    fn read_descriptor_blocks(
        &self,
        descriptor: Option<&SfcDescriptor<'_>>,
        mut read: impl FnMut(
            &mut ScriptQueryProducts,
            SourceRoot<'_>,
            &'static str,
            &SfcScriptBlock<'_>,
        ) -> Result<(), ScriptQueryRefusal>,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        Ok(if let Some(descriptor) = descriptor {
            let root = SourceRoot::new(self.source()).map_err(ScriptQueryRefusal::Frame)?;
            let mut products = ScriptQueryProducts::default();
            for (name, block) in [
                ("script", descriptor.script.as_ref()),
                ("script setup", descriptor.script_setup.as_ref()),
            ] {
                if let Some(block) = block {
                    read(&mut products, root, name, block)?;
                }
            }
            Some(products)
        } else {
            None
        })
    }
}

fn script_frame<'a>(
    root: SourceRoot<'a>,
    block: &SfcScriptBlock<'_>,
) -> Result<(SourceBlock<'a>, &'a str), ScriptQueryRefusal> {
    let text = root
        .source()
        .get(block.loc.start..block.loc.end)
        .filter(|text| *text == block.content.as_ref())
        .ok_or(ScriptQueryRefusal::DescriptorSourceMismatch)?;
    let start =
        u32::try_from(block.loc.start).map_err(|_| ScriptQueryRefusal::DescriptorSourceMismatch)?;
    let frame = root.block(text, start).map_err(ScriptQueryRefusal::Frame)?;
    let prefix = root
        .source()
        .get(..block.loc.tag_end)
        .ok_or(ScriptQueryRefusal::DescriptorSourceMismatch)?;
    Ok((frame, prefix))
}

impl ScriptQueryProducts {
    fn read_script(
        &mut self,
        root: SourceRoot<'_>,
        name: &'static str,
        block: &SfcScriptBlock<'_>,
        additional_profile: Option<SourceType>,
        mut observe: impl FnMut(&AuthoredProgram<'_>) -> Result<(), ScriptQueryRefusal>,
    ) -> Result<(), ScriptQueryRefusal> {
        let (frame, prefix) = script_frame(root, block)?;
        if frame.source().is_empty() && additional_profile.is_none() {
            let symbol =
                Self::script_block_symbol(name, block, &LineIndex::new(prefix), Vec::new());
            self.push_script_symbol(symbol);
            return Ok(());
        }
        let editor_type = Self::script_type(block.lang.as_deref());
        let symbol = {
            let mut arena = Allocator::default();
            let symbol = {
                let syntax = AuthoredProgram::parse(&arena, frame, editor_type);
                #[cfg(feature = "experimental-checker-script-queries")]
                let symbol = if additional_profile.is_some() {
                    Self::checker_script_symbol(name, block, prefix, &syntax)
                } else {
                    Self::script_symbol(name, block, &LineIndex::new(prefix), &syntax)
                };
                #[cfg(not(feature = "experimental-checker-script-queries"))]
                let symbol = Self::script_symbol(name, block, &LineIndex::new(prefix), &syntax);
                #[cfg(feature = "experimental-checker-script-queries")]
                if additional_profile.is_some() {
                    self.read_selected_specifiers(&syntax);
                } else {
                    self.read_specifiers(&syntax);
                }
                #[cfg(not(feature = "experimental-checker-script-queries"))]
                self.read_specifiers(&syntax);
                if additional_profile == Some(editor_type) {
                    observe(&syntax)?;
                }
                symbol
            };
            if let Some(profile) = additional_profile.filter(|profile| *profile != editor_type) {
                // OXC reset keeps only its last chunk. When that chunk cannot hold
                // the first AST, grow a fresh arena as a separate read would. The
                // other grammar can need a few more bytes, so reserving the observed
                // size overflowed into a doubled chunk and raised the peak.
                let used = arena.allocated_bytes();
                arena.reset();
                if arena.as_oxc().capacity() < used {
                    drop(arena);
                    arena = Allocator::default();
                }
                observe(&AuthoredProgram::parse(&arena, frame, profile))?;
            }
            symbol
        };
        self.push_script_symbol(symbol);
        Ok(())
    }

    pub(crate) fn push_script_symbol(&mut self, symbol: DocumentSymbol) {
        if self.symbols.is_empty() {
            // An SFC has at most one ordinary script and one setup script.
            self.symbols.reserve_exact(2);
        }
        self.symbols.push(symbol);
    }
}

#[cfg(all(test, feature = "micro-profiling"))]
mod profile;
#[cfg(test)]
mod tests;

#[cfg(feature = "experimental-checker-script-queries")]
mod checker;

#[cfg(feature = "experimental-script-tools")]
pub(super) mod tools;

mod native;
