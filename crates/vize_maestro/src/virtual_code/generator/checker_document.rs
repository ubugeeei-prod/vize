//! Template virtual documents are the checker’s L4 virtual TypeScript.
//!
//! Normal `.vue` files, art variants, and standalone HTML all publish that
//! document. Mappings are SFC-absolute or fragment-absolute. Nothing here
//! calls `parse_sfc`.

use vize_atelier_sfc::SfcDescriptor;
use vize_atelier_sfc::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor_with_context, analyze_sfc_descriptor_with_occurrences,
};
use vize_canon::virtual_ts::{
    VirtualTsOptions, generate_virtual_ts_with_offsets, generate_virtual_ts_with_split_offsets,
};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_l0::cstr;
use vize_relief::RootNode;

use super::super::{VirtualDocument, VirtualLanguage};

pub(super) fn template_document(
    descriptor: &SfcDescriptor<'_>,
    root: &RootNode<'_>,
    template_offset: u32,
    base_uri: &str,
    source: Option<&str>,
) -> (
    VirtualDocument,
    Option<super::occurrences::PhysicalOccurrences>,
) {
    let mut options = SfcCroquisOptions::lint_demand();
    options.analyzer_options.collect_template_expressions = true;
    let (analysis, packet) = if source.is_some() {
        analyze_sfc_descriptor_with_occurrences(descriptor, Some(root), options)
    } else {
        (
            analyze_sfc_descriptor_with_context(descriptor, Some(root), options),
            None,
        )
    };
    let packet = source.zip(packet).and_then(|(source, packet)| {
        super::occurrences::PhysicalOccurrences::from_sfc(packet, &analysis, descriptor, source)
    });
    let output = generate_virtual_ts_with_split_offsets(
        &analysis.croquis,
        analysis.script_content.as_deref(),
        Some(root),
        analysis.script_offset,
        template_offset,
        &VirtualTsOptions::default(),
        analysis.split_script_setup_offsets(descriptor),
    );
    (
        VirtualDocument::from_emission(
            cstr!("{base_uri}.__template.ts").to_string(),
            output.code.to_string(),
            VirtualLanguage::Template,
            output.mapping,
        ),
        packet,
    )
}

/// A template fragment, with an optional setup script, as the checker document.
///
/// `template_offset` is the fragment's start in the authored file. Script
/// mappings use `script_offset` the same way.
pub(super) fn fragment_document(
    script: Option<&str>,
    script_setup: bool,
    script_offset: u32,
    root: &RootNode<'_>,
    template_offset: u32,
    uri: String,
) -> VirtualDocument {
    fragment_document_demand(
        script,
        script_setup,
        script_offset,
        root,
        template_offset,
        uri,
        None,
    )
    .0
}

#[expect(
    clippy::too_many_arguments,
    reason = "same existing fragment coordinates plus optional source witness"
)]
pub(super) fn fragment_document_demand(
    script: Option<&str>,
    script_setup: bool,
    script_offset: u32,
    root: &RootNode<'_>,
    template_offset: u32,
    uri: String,
    source: Option<&str>,
) -> (
    VirtualDocument,
    Option<super::occurrences::PhysicalOccurrences>,
) {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if source.is_some() {
        analyzer = analyzer.with_binding_occurrences();
    }
    if let Some(script) = script {
        if script_setup {
            analyzer.analyze_script_setup(script);
        } else {
            analyzer.analyze_script(script);
        }
    }
    analyzer.analyze_template(root);
    let (summary, packet) = if source.is_some() {
        analyzer.finish_with_binding_occurrences()
    } else {
        (analyzer.finish(), None)
    };
    let packet = source.zip(packet).and_then(|(source, packet)| {
        super::occurrences::PhysicalOccurrences::from_fragment(
            packet,
            script_offset,
            template_offset,
            source,
        )
    });
    let output = generate_virtual_ts_with_offsets(
        &summary,
        script,
        Some(root),
        script_offset,
        template_offset,
        &VirtualTsOptions::default(),
    );
    (
        VirtualDocument::from_emission(
            uri,
            output.code.to_string(),
            VirtualLanguage::Template,
            output.mapping,
        ),
        packet,
    )
}
