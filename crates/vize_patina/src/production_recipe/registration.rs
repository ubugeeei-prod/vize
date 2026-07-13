use vize_atlas::{Compilation, RegisterProviderError, Shared};

use super::{PatinaDocumentProvider, PatinaLinterInput};
use crate::Linter;

/// Replace the configured production linter without rebuilding the graph.
pub fn install_document_linter(
    compilation: &mut Compilation,
    linter: Shared<Linter>,
) -> Result<(), vize_atlas::CompilationInputError> {
    compilation
        .set_input::<PatinaLinterInput>(linter)
        .map(|_| ())
}

/// Register one configured production Patina root.
pub fn register_document_lint_recipe(
    compilation: &mut Compilation,
    linter: Linter,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaDocumentProvider::new(linter))
}

/// Register a production Patina root backed by one shared configured linter.
pub fn register_shared_document_lint_recipe(
    compilation: &mut Compilation,
    linter: Shared<Linter>,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaDocumentProvider::from_shared(linter))
}
