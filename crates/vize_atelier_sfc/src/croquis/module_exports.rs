//! Demand-only module export admission without another script parse.

use super::analysis::{
    DescriptorAnalysisMode, DescriptorObservation, analyze_sfc_descriptor_resolved_impl,
};
use super::{SfcCroquisAnalysis, SfcCroquisOptions};
use crate::SfcDescriptor;
use vize_atelier_core::RootNode;

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
    let mut empty = false;
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
            empty_module_exports: Some(&mut empty),
            ..Default::default()
        },
    );
    if (!options.merge_scripts && descriptor.script.is_some() && descriptor.script_setup.is_some())
        || !descriptor_module_sources_are_known(descriptor)
    {
        empty = false;
    }
    (analysis, empty)
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
