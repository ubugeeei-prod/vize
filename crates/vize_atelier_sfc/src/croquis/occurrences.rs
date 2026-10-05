//! Demand-only editor facts carried through the existing descriptor analysis.

use super::analysis::{DescriptorAnalysisMode, analyze_sfc_descriptor_resolved_impl};
use super::{SfcCroquisAnalysis, SfcCroquisOptions};
use crate::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_croquis::binding_occurrences::BindingOccurrences;

/// Analyze once and return a separate owned occurrence packet. Existing options,
/// Croquis public fields, snapshots, and diagnostics are unchanged.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_occurrences(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> (SfcCroquisAnalysis, Option<BindingOccurrences>) {
    let mut packet = None;
    let analysis = analyze_sfc_descriptor_resolved_impl(
        descriptor,
        root,
        options,
        DescriptorAnalysisMode {
            options_api: false,
            legacy_vue2: false,
            include_script_content: true,
        },
        None,
        None,
        Some(&mut packet),
    );
    if options.template_is_derived
        || root.is_some_and(|root| {
            descriptor
                .template
                .as_ref()
                .is_none_or(|template| root.source != template.content.as_ref())
        })
        || descriptor.script.as_ref().is_some_and(|block| {
            block.src.is_some()
                || block
                    .lang
                    .as_deref()
                    .is_some_and(|lang| !matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        })
    {
        packet = None;
    }
    (analysis, packet)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod refusal_tests;
