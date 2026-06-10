//! Read-only queries over registered virtual files: lookup by path, ordered
//! iteration, and bidirectional position mapping between original sources and
//! their materialized virtual counterparts.

use std::path::{Path, PathBuf};

use crate::batch::SfcBlockType;

use super::{Diagnostic, OriginalPosition, VirtualFile, VirtualProject};

impl VirtualProject {
    /// Find a virtual file by its original path.
    pub fn find_by_original(&self, original_path: &Path) -> Option<&VirtualFile> {
        let virtual_path = self.original_index.get(original_path)?;
        self.virtual_files.get(virtual_path)
    }

    /// Find a virtual file by its materialized path.
    pub fn find_by_virtual(&self, virtual_path: &Path) -> Option<&VirtualFile> {
        self.virtual_files.get(virtual_path)
    }

    /// Return virtual files sorted by original path for deterministic output.
    pub fn virtual_files_sorted(&self) -> Vec<&VirtualFile> {
        let mut files: Vec<_> = self.virtual_files.values().collect();
        files.sort_by(|left, right| left.original_path.cmp(&right.original_path));
        files
    }

    /// Parser diagnostics collected while registering source files.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Original source text of a registered file, keyed by its virtual path.
    pub(crate) fn original_content_for_virtual(&self, virtual_path: &Path) -> Option<&str> {
        self.original_contents
            .get(virtual_path)
            .map(|content| content.as_str())
    }

    /// Map a virtual position to the original position.
    pub fn map_to_original(
        &self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let file = self.virtual_files.get(virtual_path)?;
        let virtual_offset =
            crate::batch::source_map::line_col_to_offset(&file.content, line, column)?;
        let (original_offset, _, block_type) =
            file.source_map.get_original_position(virtual_offset)?;
        let original_content = self.original_contents.get(&file.virtual_path)?;
        let (original_line, original_column) =
            crate::batch::source_map::offset_to_line_col(original_content, original_offset)?;

        Some(OriginalPosition {
            path: file.original_path.clone(),
            line: original_line,
            column: original_column,
            block_type,
        })
    }

    /// Map an original position to the virtual position.
    pub fn map_to_virtual(
        &self,
        original_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<(PathBuf, u32, u32)> {
        let file = self.find_by_original(original_path)?;
        let original_content = self.original_contents.get(&file.virtual_path)?;
        let original_offset =
            crate::batch::source_map::line_col_to_offset(original_content, line, column)?;
        let virtual_offset = if let Some(ref sfc_map) = file.source_map.sfc_map {
            for block in [
                SfcBlockType::ScriptSetup,
                SfcBlockType::Script,
                SfcBlockType::Template,
            ] {
                if let Some(virtual_offset) = sfc_map.get_virtual_offset(original_offset, block) {
                    let rewritten_offset = file
                        .source_map
                        .import_map
                        .get_virtual_offset(virtual_offset);
                    if let Some((virtual_line, virtual_column)) =
                        crate::batch::source_map::offset_to_line_col(
                            &file.content,
                            rewritten_offset,
                        )
                    {
                        return Some((file.virtual_path.clone(), virtual_line, virtual_column));
                    }
                }
            }
            return None;
        } else {
            file.source_map
                .import_map
                .get_virtual_offset(original_offset)
        };

        let (virtual_line, virtual_column) =
            crate::batch::source_map::offset_to_line_col(&file.content, virtual_offset)?;
        Some((file.virtual_path.clone(), virtual_line, virtual_column))
    }

    /// Get the number of registered files.
    pub fn file_count(&self) -> usize {
        self.virtual_files.len()
    }

    /// Check if the project has any files.
    pub fn is_empty(&self) -> bool {
        self.virtual_files.is_empty()
    }
}
