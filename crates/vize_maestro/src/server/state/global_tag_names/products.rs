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
        if !supported_declaration(self.uri()) {
            return Err(DeclarationToolRefusal::Path);
        }
        let source = self.source();
        if source.is_empty() {
            return Ok(DeclarationToolProducts {
                checker: DeclarationCheckerProducts {
                    typescript: String::default(),
                    reexports_vue_template_ref: false,
                },
                component_tags: Vec::new(),
            });
        }
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

fn supported_declaration(uri: &tower_lsp::lsp_types::Url) -> bool {
    let supported = |path: &std::path::Path| {
        DeclarationCheckerProducts::supports_path(path)
            && !vize_carton::path::is_git_metadata_path(path)
    };
    // A local Unix file URL with no escaped bytes already contains its
    // absolute filesystem path. Encoded paths and authorities use the URL
    // library's native validation and platform conversion.
    #[cfg(unix)]
    if uri.scheme() == "file" && uri.host().is_none() && !uri.path().as_bytes().contains(&b'%') {
        return supported(std::path::Path::new(uri.path()));
    }
    uri.to_file_path().is_ok_and(|path| supported(&path))
}

#[cfg(test)]
mod tests;
