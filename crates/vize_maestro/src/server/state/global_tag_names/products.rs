//! Read a declaration's checker products and its completion contribution together.
use super::{ServerState, names_from_program};
use crate::source_project::{SnapshotRefusal, SourceSnapshot};
use vize_canon::batch::declaration_products::DeclarationCheckerProducts;
use vize_l0::{Allocator, SourceFrameError, SourceRoot, String};
use vize_l1::embed::declaration_program::DeclarationProgram;

#[derive(Debug, PartialEq, Eq)]
pub struct DeclarationToolProducts {
    pub checker: DeclarationCheckerProducts,
    /// Sorted, deduplicated component tags contributed by this declaration.
    pub component_tags: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeclarationToolRefusal {
    Host(SnapshotRefusal),
    Path,
    Frame(SourceFrameError),
}

impl SourceSnapshot {
    /// Read Canon's declaration projection and Vue re-export and Maestro's global
    /// component names from one parse of the current open declaration.
    pub fn declaration_tool_products(
        &self,
        state: &ServerState,
    ) -> Result<DeclarationToolProducts, DeclarationToolRefusal> {
        self.check_current(&state.documents)
            .map_err(DeclarationToolRefusal::Host)?;
        let path = self
            .uri()
            .to_file_path()
            .map_err(|()| DeclarationToolRefusal::Path)?;
        if !DeclarationCheckerProducts::supports_path(&path)
            || vize_carton::path::is_git_metadata_path(&path)
        {
            return Err(DeclarationToolRefusal::Path);
        }
        let source = self.source();
        let root = SourceRoot::new(source).map_err(DeclarationToolRefusal::Frame)?;
        let arena = Allocator::default();
        let declaration = DeclarationProgram::parse(&arena, root);
        let component_tags = if source.len() <= 4 * 1024 * 1024
            && (source.contains("GlobalComponents") || source.contains('\\'))
        {
            declaration
                .clean()
                .map_or_else(Vec::new, names_from_program)
        } else {
            Vec::new()
        };
        let products = DeclarationToolProducts {
            component_tags,
            checker: DeclarationCheckerProducts::from_program(&declaration),
        };
        self.check_current(&state.documents)
            .map_err(DeclarationToolRefusal::Host)?;
        Ok(products)
    }
}

#[cfg(test)]
mod tests;
