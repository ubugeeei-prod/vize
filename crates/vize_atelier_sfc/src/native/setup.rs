//! Same original SFC custody supplies script and render fragments together.

use super::NativeSfcCompileError;
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::lang::js::VueSetup;
use vize_l3::decision::dom::vue::build_vue_render_decisions;
use vize_l4::{
    module::{ModuleParts, ScriptPart, setup::emit_setup},
    targets::dom::emit_vue,
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
        .setup()
        .ok_or(NativeSfcCompileError::Orchestration)?;
    let original = observation
        .scripts()
        .first()
        .and_then(|script| script.syntax())
        .and_then(|syntax| syntax.admitted_program())
        .ok_or(NativeSfcCompileError::Orchestration)?;
    let setup = VueSetup::checked(admitted.file().file(), script, original)
        .map_err(NativeSfcCompileError::ScriptSetup)?;
    let prepared = emit_setup::<L>(&setup).map_err(NativeSfcCompileError::SetupEmission)?;
    let analysis =
        build_vue_render_decisions(setup.exposure()).map_err(NativeSfcCompileError::Analysis)?;
    let render = emit_vue::<L>(&analysis).map_err(NativeSfcCompileError::Dom)?;
    parts.script = Some(ScriptPart::Body(prepared));
    parts.render = Some(render);
    Ok(())
}

#[cfg(test)]
mod tests;
