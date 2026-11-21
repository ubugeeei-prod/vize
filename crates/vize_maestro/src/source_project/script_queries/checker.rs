//! Compose the editor's script readers with the checker's SFC syntax diagnostics.
use super::{ScriptQueryProducts, ScriptQueryRefusal, ServerState, SourceSnapshot};
use vize_canon::{ScriptDiagnosticProfile, SfcTypeDiagnostic};

impl SourceSnapshot {
    /// Return the script outline, file-rename candidates and Canon's block syntax
    /// diagnostics for this revision. Each distinct requested grammar parses the
    /// authored block once. SFC generic parameters belong to the enclosing tag.
    pub fn script_query_products_with_diagnostics(
        &self,
        state: &ServerState,
    ) -> Result<Option<(ScriptQueryProducts, Vec<SfcTypeDiagnostic>)>, ScriptQueryRefusal> {
        let mut diagnostics = Vec::new();
        let editor = self.with_script_blocks(state, |out, root, name, block| {
            let checker = ScriptDiagnosticProfile::for_language(block.lang.as_deref());
            out.read_script(root, name, block, Some(checker.source_type()), |syntax| {
                diagnostics.extend(
                    checker
                        .diagnostics(syntax)
                        .map_err(ScriptQueryRefusal::Checker)?,
                );
                Ok(())
            })
        })?;
        Ok(editor.map(|products| (products, diagnostics)))
    }
}

#[cfg(test)]
mod tests;
