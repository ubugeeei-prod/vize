//! Original compiler-owned script Programs can serve the editor on the same source.
use super::{ScriptQueryProducts, ScriptQueryRefusal, ServerState, SourceSnapshot, script_frame};
use core::ops::ControlFlow;
use vize_atelier_sfc::SfcScriptBlock;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::authored_program::AuthoredProgram;
use vize_l1::{container::vue::ScriptRole, embed::syntax::EmbedSyntax};
use vize_l1_to_l2::native_file::NativeSfcObservation;

impl SourceSnapshot {
    /// Collect the ordinary outline and file-rename candidates after native SFC lowering.
    /// A broken sibling can retain its own editor recovery behavior while another
    /// block supplies its actual Program. The observation must borrow this snapshot.
    pub fn script_query_products_from(
        &self,
        state: &ServerState,
        owner: &NativeSfcObservation<'_>,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        self.with_native_script_blocks(state, owner, |out, root, name, block| {
            if out.read_retained_script(root, name, block, owner)? {
                return Ok(());
            }
            let (frame, prefix) = script_frame(root, block)?;
            let symbol = {
                let arena = Allocator::default();
                let syntax = AuthoredProgram::parse(
                    &arena,
                    frame,
                    ScriptQueryProducts::script_type(block.lang.as_deref()),
                );
                let symbol = ScriptQueryProducts::script_symbol(name, block, prefix, &syntax);
                out.read_authored_specifiers(&syntax);
                symbol
            };
            out.push_script_symbol(symbol);
            Ok(())
        })
    }

    fn with_native_script_blocks(
        &self,
        state: &ServerState,
        owner: &NativeSfcObservation<'_>,
        read: impl FnMut(
            &mut ScriptQueryProducts,
            SourceRoot<'_>,
            &'static str,
            &SfcScriptBlock<'_>,
        ) -> Result<(), ScriptQueryRefusal>,
    ) -> Result<Option<ScriptQueryProducts>, ScriptQueryRefusal> {
        if !core::ptr::eq(self.source(), owner.l1ir().source()) {
            self.check_current(&state.documents)
                .map_err(ScriptQueryRefusal::Host)?;
            return Err(ScriptQueryRefusal::DescriptorSourceMismatch);
        }
        let descriptor = self.with_current_script_descriptor(state, |parsed| {
            let Some(descriptor) = parsed else {
                return Ok(ControlFlow::Break(None));
            };
            if descriptor.script.is_none() && descriptor.script_setup.is_none() {
                // The entire empty answer is known while this revision's document is locked.
                SourceRoot::new(self.source()).map_err(ScriptQueryRefusal::Frame)?;
                return Ok(ControlFlow::Break(Some(ScriptQueryProducts::default())));
            }
            Ok(ControlFlow::Continue(descriptor))
        })?;
        let descriptor = match descriptor {
            ControlFlow::Break(result) => return Ok(result),
            ControlFlow::Continue(descriptor) => descriptor,
        };
        let result = self.read_descriptor_blocks(Some(&descriptor), read)?;
        self.check_current(&state.documents)
            .map_err(ScriptQueryRefusal::Host)?;
        Ok(result)
    }
}

impl ScriptQueryProducts {
    fn read_retained_script(
        &mut self,
        root: SourceRoot<'_>,
        name: &'static str,
        block: &SfcScriptBlock<'_>,
        owner: &NativeSfcObservation<'_>,
    ) -> Result<bool, ScriptQueryRefusal> {
        let (frame, prefix) = script_frame(root, block)?;
        if frame.source().is_empty() {
            self.push_script_symbol(Self::retained_empty_symbol(name, block, prefix));
            return Ok(true);
        }
        if let Some(syntax) = matching_script(owner, block)
            && let Some(program) = syntax.program()
            && core::ptr::eq(frame.source(), syntax.source().text())
        {
            let outline = syntax.diagnostics().next().is_none().then_some(program);
            self.push_script_symbol(Self::symbol_from_program(name, block, prefix, outline));
            let visit_nested =
                outline.is_none() || Self::has_specifier_argument_list(frame.source().as_bytes());
            self.read_retained_specifiers(program, frame.start(), visit_nested);
            return Ok(true);
        }
        Ok(false)
    }
}

fn matching_script<'o, 'a>(
    owner: &'o NativeSfcObservation<'a>,
    block: &SfcScriptBlock<'_>,
) -> Option<&'o EmbedSyntax<'a>> {
    let role = if block.setup {
        ScriptRole::Setup
    } else {
        ScriptRole::Ordinary
    };
    let observed = owner
        .scripts()
        .iter()
        .find(|script| script.role() == role)?;
    let span = observed.block().span();
    let syntax = observed.syntax()?;
    (span.start as usize == block.loc.start
        && span.end as usize == block.loc.end
        && syntax.source_type() == ScriptQueryProducts::script_type(block.lang.as_deref()))
    .then_some(syntax)
}

#[cfg(test)]
mod tests;

#[cfg(feature = "experimental-checker-script-queries")]
mod checker;
