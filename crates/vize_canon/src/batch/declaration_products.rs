//! Checker products from one original declaration, retaining each reader's recovery policy.
use super::{declaration_path, vue_template_ref_module};
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

impl DeclarationCheckerProducts {
    #[must_use]
    pub fn supports_path(path: &Path) -> bool {
        declaration_path::is_declaration_file(path)
    }

    #[must_use]
    pub fn from_program(declaration: &DeclarationProgram<'_>) -> Self {
        let text = declaration.source();
        let reexports_vue_template_ref = (text.contains("useTemplateRef")
            || text.contains("export *"))
            && declaration
                .recovered()
                .is_some_and(vue_template_ref_module::exports_template_ref);
        let typescript = match declaration.clean() {
            Some(program) => declaration_path::project_program(text, program),
            None => text.to_compact_string(),
        };
        Self {
            typescript,
            reexports_vue_template_ref,
        }
    }
}
