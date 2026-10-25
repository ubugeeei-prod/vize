//! Build effect facts from original block grammar and binding ownership.

use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::{EffectGraphScript, EffectGraphSummary, build_effect_graph_from_sfc_scripts};

pub(super) fn from_descriptor(descriptor: &SfcDescriptor<'_>) -> EffectGraphSummary {
    let script = descriptor
        .script
        .as_ref()
        .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref()));
    let setup = descriptor
        .script_setup
        .as_ref()
        .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref()));
    build_effect_graph_from_sfc_scripts(script, setup).summary()
}

#[cfg(test)]
mod tests;
