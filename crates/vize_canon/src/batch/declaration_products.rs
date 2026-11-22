//! Checker products from one original declaration, retaining each reader's recovery policy.
use super::declaration_path::{self, observed};
use std::path::Path;
use vize_l0::{String, ToCompactString};
use vize_l1::embed::declaration_program::DeclarationProgram;

/// Published by the checker for the original ambient module.
#[derive(Debug, PartialEq, Eq)]
pub struct DeclarationCheckerProducts {
    /// The complete source projected for a plain TypeScript host.
    pub typescript: String,
    /// Whether a value import of `useTemplateRef` resolves to Vue's own export.
    pub reexports_vue_template_ref: bool,
}

/// The shipping projection places the next ambient modifier on an export
/// assignment. The corrected candidate is available for differential inspection.
#[derive(Debug, PartialEq, Eq)]
pub struct DeclarationProjectionConflict {
    /// Original UTF-8 byte offset shared by the two insertions.
    pub offset: usize,
    pub corrected: DeclarationCheckerProducts,
}

impl DeclarationCheckerProducts {
    #[must_use]
    pub fn supports_path(path: &Path) -> bool {
        declaration_path::is_declaration_file(path)
    }

    pub fn from_program(
        declaration: &DeclarationProgram<'_>,
    ) -> Result<Self, DeclarationProjectionConflict> {
        let text = declaration.source();
        let inspect_exports = text.contains("useTemplateRef") || text.contains("export *");
        if let Some(program) = declaration.clean() {
            return observed::products(text, program, inspect_exports);
        }
        Ok(Self {
            typescript: text.to_compact_string(),
            reexports_vue_template_ref: inspect_exports
                && declaration.recovered().is_some_and(|program| {
                    program.body.iter().any(observed::vue_template_ref_export)
                }),
        })
    }
}
