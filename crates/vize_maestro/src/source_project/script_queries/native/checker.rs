//! Canon diagnostics share the original compiler parse when its exact profile is clean.
use super::{
    AuthoredProgram, NativeSfcObservation, ScriptQueryProducts, ScriptQueryRefusal, ServerState,
    SourceSnapshot, matching_script, script_frame,
};
use vize_canon::{ScriptDiagnosticProfile, SfcTypeDiagnostic};
use vize_l0::{Allocator, SourceBlock};
use vize_l1::embed::syntax::EmbedSyntax;

impl SourceSnapshot {
    /// Read editor script products and Canon's syntax diagnostics after native
    /// lowering. A retained Program with this exact requested grammar and no
    /// diagnostics supplies the checker's complete empty answer. Other profiles
    /// and local holes keep their existing parser and recovery behavior. No
    /// complete native File is required or admitted by this syntax-only read.
    pub fn script_query_products_with_diagnostics_from(
        &self,
        state: &ServerState,
        owner: &NativeSfcObservation<'_>,
    ) -> Result<Option<(ScriptQueryProducts, Vec<SfcTypeDiagnostic>)>, ScriptQueryRefusal> {
        let mut diagnostics = Vec::new();
        let editor = self.with_native_script_blocks(state, owner, |out, root, name, block| {
            let (frame, prefix) = script_frame(root, block)?;
            let checker = ScriptDiagnosticProfile::for_language(block.lang.as_deref());
            if !out.read_retained_script(frame, prefix, name, block, owner) {
                // Preserve the existing shared-grammar schedule when native
                // syntax is unavailable: the editor and checker still parse once.
                return out.read_script_frame(
                    name,
                    block,
                    frame,
                    prefix,
                    Some(checker.source_type()),
                    |syntax| {
                        diagnostics.extend(
                            checker
                                .diagnostics(syntax)
                                .map_err(ScriptQueryRefusal::Checker)?,
                        );
                        Ok(())
                    },
                );
            }
            if frame.source().is_empty()
                || matching_script(owner, block)
                    .is_some_and(|syntax| clean_checker_program(syntax, frame, checker))
            {
                return Ok(());
            }
            let arena = Allocator::default();
            let syntax = AuthoredProgram::parse(&arena, frame, checker.source_type());
            diagnostics.extend(
                checker
                    .diagnostics(&syntax)
                    .map_err(ScriptQueryRefusal::Checker)?,
            );
            Ok(())
        })?;
        Ok(editor.map(|products| (products, diagnostics)))
    }
}

fn clean_checker_program(
    syntax: &EmbedSyntax<'_>,
    block: SourceBlock<'_>,
    checker: ScriptDiagnosticProfile,
) -> bool {
    syntax.source_type() == checker.source_type()
        && core::ptr::eq(syntax.source().text(), block.source())
        && syntax.program().is_some()
        && syntax.diagnostics().next().is_none()
}

#[cfg(test)]
mod tests;
