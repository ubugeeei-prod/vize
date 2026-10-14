//! Demand-only module export admission without another script parse.

use super::analysis::{
    DescriptorAnalysisMode, DescriptorObservation, analyze_sfc_descriptor_resolved_impl,
};
use super::{SfcCroquisAnalysis, SfcCroquisOptions};
use crate::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_croquis::script_parser::AuthoredModuleExport;

/// Analyze the same original blocks and return separate empty-export metadata.
/// Existing public option/result models and their serialized forms are unchanged.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_empty_module_exports(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> (SfcCroquisAnalysis, bool) {
    let (analysis, exports) = analyze_sfc_descriptor_with_module_exports(
        descriptor,
        root,
        options,
        options_api,
        legacy_vue2,
    );
    (analysis, exports.is_some_and(|exports| exports.is_empty()))
}

/// Analyze the same original blocks and return every authored named export of
/// the SFC module, or `None` when its export set is not provable from the
/// original scripts alone. A star re-export of the component can then name
/// exactly these exports instead of also exposing generated declarations.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_module_exports(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> (SfcCroquisAnalysis, Option<Vec<AuthoredModuleExport>>) {
    let mut exports = None;
    let analysis = analyze_sfc_descriptor_resolved_impl::<false>(
        descriptor,
        root,
        options,
        DescriptorAnalysisMode {
            options_api,
            legacy_vue2,
            include_script_content: true,
        },
        None,
        None,
        DescriptorObservation {
            module_exports: Some(&mut exports),
            ..Default::default()
        },
    );
    if (!options.merge_scripts && descriptor.script.is_some() && descriptor.script_setup.is_some())
        || !descriptor_module_sources_are_known(descriptor)
    {
        exports = None;
    }
    (analysis, exports)
}

/// Preserve admission before a caller prepares or drops source blocks.
#[doc(hidden)]
pub fn descriptor_module_sources_are_known(descriptor: &SfcDescriptor<'_>) -> bool {
    [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
        .all(|block| {
            block.src.is_none()
                && block.lang.as_deref().is_none_or(|lang| {
                    matches!(lang.trim(), "js" | "ts" | "jsx" | "tsx" | "mjs" | "mts")
                })
        })
}

#[cfg(test)]
mod tests;
