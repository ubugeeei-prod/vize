//! Private source origins and binding authority for authored module emission.

use vize_atelier_core::BindingMetadata;
use vize_croquis::Croquis;
use vize_l0::String;

/// A backend map describes its render body; SSR puts an unmapped preamble in
/// front of that body. Keep the exact producer offset instead of rewriting maps.
#[derive(Default)]
pub(crate) struct RootSourceMap {
    pub(crate) json: Option<String>,
    pub(crate) code_start: usize,
}

/// Convert the actual Croquis metadata once per authored module. Binding kinds
/// and aliases retain their producer's values; names alone never invent kinds.
pub(crate) fn bindings(analysis: &Croquis) -> BindingMetadata {
    BindingMetadata {
        bindings: analysis
            .bindings
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str().into(), *kind))
            .collect(),
        props_aliases: analysis
            .bindings
            .props_aliases
            .iter()
            .map(|(local, prop)| (local.as_str().into(), prop.as_str().into()))
            .collect(),
        is_script_setup: analysis.bindings.is_script_setup,
    }
}
