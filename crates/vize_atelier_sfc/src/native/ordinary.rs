//! The actual sole ordinary descriptor joins its original complete File receipt.

use super::NativeSfcCompileError;
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::lang::js::VueOrdinaryEmpty;
use vize_l3::decision::build_dom_file_decisions;
use vize_l4::{
    module::{ModuleParts, ScriptPart, ordinary::emit_ordinary_empty},
    targets::dom::emit_file,
    write::LinkSink,
};

pub(super) fn emit<L: LinkSink>(
    admitted: &NativeSfc<'_, '_>,
    parts: &mut ModuleParts<'_, L>,
) -> Result<(), NativeSfcCompileError> {
    let observation = admitted.observation();
    let descriptor = observation
        .descriptor()
        .admitted()
        .map_err(|_| NativeSfcCompileError::Descriptor)?;
    let script = descriptor
        .ordinary()
        .ok_or(NativeSfcCompileError::Orchestration)?;
    // A per-script File receipt cannot prove an omitted descriptor sibling.
    if descriptor.setup().is_some() || observation.scripts().len() != 1 {
        return Err(NativeSfcCompileError::ScriptCompilationUnavailable {
            container_index: script.container_index(),
            span: script.block().span(),
        });
    }
    let original = observation
        .scripts()
        .first()
        .and_then(|script| script.syntax())
        .and_then(|syntax| syntax.admitted_program())
        .ok_or(NativeSfcCompileError::Orchestration)?;
    let ordinary = VueOrdinaryEmpty::checked(admitted.file().file(), script, original)
        .map_err(NativeSfcCompileError::ScriptOrdinary)?;
    let prepared =
        emit_ordinary_empty::<L>(&ordinary).map_err(NativeSfcCompileError::OrdinaryEmission)?;
    let analysis =
        build_dom_file_decisions(ordinary.file()).map_err(NativeSfcCompileError::Analysis)?;
    let render = emit_file::<L>(&analysis).map_err(NativeSfcCompileError::Dom)?;
    parts.script = Some(ScriptPart::Body(prepared));
    parts.render = Some(render);
    Ok(())
}

#[cfg(test)]
mod tests;
