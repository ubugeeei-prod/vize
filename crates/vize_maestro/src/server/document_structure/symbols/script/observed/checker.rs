//! Script modules without declaration children need only their two tag positions.
use super::{
    AuthoredProgram, DocumentSymbol, LineIndex, ScriptQueryProducts, SfcScriptBlock, Statement,
    tag_ranges,
};

impl ScriptQueryProducts {
    pub(crate) fn checker_script_symbol(
        name: &str,
        block: &SfcScriptBlock<'_>,
        prefix: &str,
        syntax: &AuthoredProgram<'_>,
    ) -> DocumentSymbol {
        let parsed = syntax.parsed();
        let declarations = !parsed.panicked
            && parsed.diagnostics.is_empty()
            && parsed.program.body.iter().any(|statement| match statement {
                Statement::ExportNamedDeclaration(export) => export.declaration.is_some(),
                other => other.as_declaration().is_some(),
            });
        if !declarations && let Some((range, selection_range)) = tag_ranges(block, prefix) {
            return Self::module_symbol(name, block, (range, selection_range), Vec::new());
        }
        Self::script_symbol(name, block, &LineIndex::new(prefix), syntax)
    }
}

#[cfg(feature = "experimental-script-tools")]
mod tools;
