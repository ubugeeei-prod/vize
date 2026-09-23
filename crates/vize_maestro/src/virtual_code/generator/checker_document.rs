//! Template virtual documents are the checker’s S4 virtual TypeScript.
//!
//! Normal `.vue` files, art variants, and standalone HTML all publish that
//! document. Mappings are SFC-absolute or fragment-absolute. Nothing here
//! calls `parse_sfc`.

use vize_atelier_sfc::SfcDescriptor;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context};
use vize_canon::virtual_ts::{
    VirtualTsOptions, generate_virtual_ts_with_offsets, generate_virtual_ts_with_split_offsets,
};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::RootNode;
use vize_s0::cstr;

use super::super::{VirtualDocument, VirtualLanguage};

pub(super) fn template_document(
    descriptor: &SfcDescriptor<'_>,
    root: &RootNode<'_>,
    template_offset: u32,
    base_uri: &str,
) -> VirtualDocument {
    let mut options = SfcCroquisOptions::lint_demand();
    options.analyzer_options.collect_template_expressions = true;
    let analysis = analyze_sfc_descriptor_with_context(descriptor, Some(root), options);
    let output = generate_virtual_ts_with_split_offsets(
        &analysis.croquis,
        analysis.script_content.as_deref(),
        Some(root),
        analysis.script_offset,
        template_offset,
        &VirtualTsOptions::default(),
        analysis.split_script_setup_offsets(descriptor),
    );
    VirtualDocument::from_emission(
        cstr!("{base_uri}.__template.ts").to_string(),
        output.code.to_string(),
        VirtualLanguage::Template,
        output.mapping,
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
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if let Some(script) = script {
        if script_setup {
            analyzer.analyze_script_setup(script);
        } else {
            analyzer.analyze_script(script);
        }
    }
    analyzer.analyze_template(root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets(
        &summary,
        script,
        Some(root),
        script_offset,
        template_offset,
        &VirtualTsOptions::default(),
    );
    VirtualDocument::from_emission(
        uri,
        output.code.to_string(),
        VirtualLanguage::Template,
        output.mapping,
    )
}
