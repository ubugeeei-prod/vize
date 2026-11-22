//! Compiler macro-shadow bindings and project reactive edges from the same script declarations.
use crate::{SfcDescriptor, SfcScriptBlock};
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::{
    EffectGraph, EffectGraphScript,
    effect_graph::{ObservedEffectError, ObservedEffectScript, visit_sfc_effect_graph},
};
use vize_l1::container::vue::ScriptRole;
use vize_l1::embed::authored_program::AuthoredProgram;
use vize_l2::query::script::{DeclarationExport, ScriptDeclarationKind};

#[derive(Debug)]
pub struct ScriptBindingProducts {
    /// Runtime bindings that shadow compiler macros in the ordinary script.
    pub ordinary: FxHashSet<CompactString>,
    /// Runtime bindings that shadow compiler macros in the setup script.
    pub setup: FxHashSet<CompactString>,
    /// Reactive dependencies in the project's ordinary and setup scope order.
    pub effects: EffectGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptBindingError {
    Selection,
    ExternalScript,
    Effect(ObservedEffectError),
}

/// Project the descriptor's inline scripts into compiler shadow sets and the
/// project effect graph. The original Program for each selected role is required.
pub fn script_binding_products(
    descriptor: &SfcDescriptor<'_>,
    ordinary: Option<&AuthoredProgram<'_>>,
    setup: Option<&AuthoredProgram<'_>>,
) -> Result<ScriptBindingProducts, ScriptBindingError> {
    let ordinary = selected(descriptor, descriptor.script.as_ref(), ordinary, false)?;
    let setup = selected(descriptor, descriptor.script_setup.as_ref(), setup, true)?;
    let mut ordinary_names = FxHashSet::default();
    let mut setup_names = FxHashSet::default();
    let effects = visit_sfc_effect_graph(ordinary.as_ref(), setup.as_ref(), |role, binding| {
        if matches!(
            binding.export,
            DeclarationExport::Local | DeclarationExport::Named
        ) && matches!(
            binding.kind,
            ScriptDeclarationKind::Import { type_only: false }
                | ScriptDeclarationKind::Variable(_)
                | ScriptDeclarationKind::Function
                | ScriptDeclarationKind::Class
        ) {
            let names = match role {
                ScriptRole::Ordinary => &mut ordinary_names,
                ScriptRole::Setup => &mut setup_names,
            };
            names.insert(CompactString::new(binding.name));
        }
    })
    .map_err(ScriptBindingError::Effect)?;
    Ok(ScriptBindingProducts {
        ordinary: ordinary_names,
        setup: setup_names,
        effects,
    })
}

fn selected<'p, 'a>(
    descriptor: &SfcDescriptor<'_>,
    block: Option<&SfcScriptBlock<'_>>,
    program: Option<&'p AuthoredProgram<'a>>,
    setup: bool,
) -> Result<Option<ObservedEffectScript<'p, 'a>>, ScriptBindingError> {
    let (block, program) = match (block, program) {
        (None, None) => return Ok(None),
        (Some(block), Some(program)) => (block, program),
        _ => return Err(ScriptBindingError::Selection),
    };
    if block.src.is_some() {
        return Err(ScriptBindingError::ExternalScript);
    }
    let frame = program.block();
    if block.setup != setup
        || !core::ptr::eq(descriptor.source.as_ref(), frame.root_source())
        || block.loc.start != frame.start() as usize
        || block.loc.end != frame.end() as usize
        || !(core::ptr::eq(block.content.as_ref(), frame.source())
            || block.content.as_ref() == frame.source())
    {
        return Err(ScriptBindingError::Selection);
    }
    EffectGraphScript::new(frame.source(), block.lang.as_deref())
        .with_original(program)
        .map(Some)
        .map_err(ScriptBindingError::Effect)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
