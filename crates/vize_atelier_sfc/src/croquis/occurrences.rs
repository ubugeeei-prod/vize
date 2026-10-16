//! Demand-only editor facts carried through the existing descriptor analysis.

use super::analysis::{
    DescriptorAnalysisMode, DescriptorObservation, analyze_sfc_descriptor_resolved_impl,
};
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
    analyze_sfc_descriptor_with_occurrences_and_template_reuse(descriptor, root, options, None)
}

/// Preserve the same occurrence relation while reusing demanded original
/// grammar owners. The reuse packet cannot seed or replace editor witnesses.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_occurrences_and_template_reuse(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    reuse: Option<vize_croquis::drawer::TemplateSemanticReuse>,
) -> (SfcCroquisAnalysis, Option<BindingOccurrences>) {
    let mut packet = None;
    let mut analysis = analyze_sfc_descriptor_resolved_impl::<true>(
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
        DescriptorObservation {
            occurrences: Some(&mut packet),
            template_semantic_reuse: reuse,
            ..Default::default()
        },
    );
    if options.template_is_derived
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|block| block.attrs.contains_key("generic"))
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
        // Setup generic attributes have no identifier AST in the script view.
        packet = None;
    }
    let component_reads = !options.template_is_derived
        && options.analyzer_options.analyze_script
        && options.analyzer_options.analyze_template_scopes
        && options.analyzer_options.detect_undefined
        && descriptor.script_setup.as_ref().is_some_and(|block| {
            block.src.is_none()
                && !block.attrs.contains_key("generic")
                && block
                    .lang
                    .as_deref()
                    .is_none_or(|lang| matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        })
        && descriptor.script.as_ref().is_none_or(|block| {
            block.src.is_none()
                && block
                    .lang
                    .as_deref()
                    .is_none_or(|lang| matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        })
        && root.is_some_and(|root| {
            descriptor.template.as_ref().is_some_and(|block| {
                block.src.is_none()
                    && block.lang.as_deref().is_none_or(|lang| lang == "html")
                    && root.source == block.content.as_ref()
            })
        });
    analysis
        .croquis
        .qualify_authored_component_reads(component_reads);
    (analysis, packet)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod refusal_tests;
