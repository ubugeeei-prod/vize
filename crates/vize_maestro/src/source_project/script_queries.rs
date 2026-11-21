//! Script syntax queries over the current editor's immutable source and descriptor.
#![expect(
    clippy::disallowed_types,
    reason = "move the original file-rename visitor's owned string without reallocating it"
)]

use super::{SnapshotRefusal, SourceSnapshot};
use crate::server::ServerState;
use tower_lsp::lsp_types::DocumentSymbol;
use vize_atelier_sfc::SfcScriptBlock;
use vize_croquis::sfc::SfcDescriptor;
use vize_l0::{Allocator, SourceFrameError, SourceRoot, Span, line_index::LineIndex};
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
}

impl SourceSnapshot {
    /// Read the script outline and the file-rename module specifiers together.
    /// The editor descriptor selects the physical blocks. Each original script
    /// is parsed once with its original JS, JSX, TS or TSX grammar. Concurrent
    /// edits can proceed during parsing and invalidate publication.
    pub fn script_query_products(
        &self,
        state: &ServerState,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        let parsed = self
            .with_current(&state.documents, |_| {
                if self.language_id() != "vue" || crate::utils::is_jsx_path(self.uri().path()) {
                    return Err(ScriptQueryRefusal::Language);
                }
                Ok(state.sfc_descriptor(self.uri(), self.source()))
            })
            .map_err(ScriptQueryRefusal::Host)??;
        let result = parsed
            .as_deref()
            .map(|descriptor| ScriptQueryProducts::from_descriptor(self.source(), descriptor))
            .transpose()?;
        self.check_current(&state.documents)
            .map_err(ScriptQueryRefusal::Host)?;
        Ok(result)
    }
}

impl ScriptQueryProducts {
    fn from_descriptor(
        source: &str,
        descriptor: &SfcDescriptor<'_>,
    ) -> Result<Self, ScriptQueryRefusal> {
        let root = SourceRoot::new(source).map_err(ScriptQueryRefusal::Frame)?;
        let roles = [
            ("script", descriptor.script.as_ref()),
            ("script setup", descriptor.script_setup.as_ref()),
        ];
        let mut out = Self::default();
        for (name, block) in roles {
            if let Some(block) = block {
                out.read_script(root, name, block)?;
            }
        }
        Ok(out)
    }

    fn read_script(
        &mut self,
        root: SourceRoot<'_>,
        name: &'static str,
        block: &SfcScriptBlock<'_>,
    ) -> Result<(), ScriptQueryRefusal> {
        let text = root
            .source()
            .get(block.loc.start..block.loc.end)
            .filter(|text| *text == block.content.as_ref())
            .ok_or(ScriptQueryRefusal::DescriptorSourceMismatch)?;
        let start = u32::try_from(block.loc.start)
            .map_err(|_| ScriptQueryRefusal::DescriptorSourceMismatch)?;
        let frame = root.block(text, start).map_err(ScriptQueryRefusal::Frame)?;
        let prefix = root
            .source()
            .get(..block.loc.tag_end)
            .ok_or(ScriptQueryRefusal::DescriptorSourceMismatch)?;
        let symbol = {
            let arena = Allocator::default();
            let syntax =
                AuthoredProgram::parse(&arena, frame, Self::script_type(block.lang.as_deref()));
            let symbol = Self::script_symbol(name, block, &LineIndex::new(prefix), &syntax);
            self.read_specifiers(&syntax);
            symbol
        };
        self.symbols.push(symbol);
        Ok(())
    }
}

#[cfg(all(test, feature = "micro-profiling"))]
mod profile;
#[cfg(test)]
mod tests;
