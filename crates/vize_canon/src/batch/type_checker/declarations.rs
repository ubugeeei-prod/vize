//! Declaration-emit inputs and outputs for [`super::BatchTypeChecker`].

use std::path::PathBuf;

use vize_carton::String;

/// Options for declaration emit.
#[derive(Debug, Clone)]
pub struct DeclarationEmitOptions {
    /// Output directory where emitted `.d.ts` files should be written.
    pub out_dir: PathBuf,
    /// Whether declaration maps should be emitted as well.
    pub declaration_map: bool,
}

impl DeclarationEmitOptions {
    /// Create declaration emit options for the given output directory.
    pub fn new(out_dir: PathBuf) -> Self {
        Self {
            out_dir,
            declaration_map: false,
        }
    }

    /// Enable or disable declaration map emit.
    pub fn with_declaration_map(mut self, declaration_map: bool) -> Self {
        self.declaration_map = declaration_map;
        self
    }
}

/// A single emitted declaration file.
#[derive(Debug, Clone)]
pub struct DeclarationOutput {
    /// Absolute emitted file path.
    pub path: PathBuf,
    /// Emitted file content.
    pub content: String,
}

/// Result of declaration emit.
#[derive(Debug, Clone, Default)]
pub struct DeclarationEmitResult {
    /// Emitted declaration files.
    pub files: Vec<DeclarationOutput>,
}
