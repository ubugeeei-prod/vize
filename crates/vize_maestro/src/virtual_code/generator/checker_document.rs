//! The SFC template virtual document is the checker’s S4 virtual TypeScript.
//!
//! Art and standalone HTML still use the expression generator. A normal `.vue`
//! file publishes the same document `vize check` emits, with SFC-absolute
//! mappings and no extra `parse_sfc` on the request path.

use vize_atelier_sfc::SfcDescriptor;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_relief::RootNode;
use vize_s0::cstr;

use super::super::{VirtualDocument, VirtualLanguage};

pub(super) fn template_document(
    descriptor: &SfcDescriptor<'_>,
    root: &RootNode<'_>,
    base_uri: &str,
) -> VirtualDocument {
    let template = descriptor
        .template
        .as_ref()
        .expect("template document is only built for a template block");
    let mut options = SfcCroquisOptions::for_lint();
    options.analyzer_options.collect_template_expressions = true;
    let analysis = analyze_sfc_descriptor(descriptor, Some(root), options);
    let script = descriptor
        .script_setup
        .as_ref()
        .or(descriptor.script.as_ref());
    let output = generate_virtual_ts_with_offsets(
        &analysis,
        script.map(|block| block.content.as_ref()),
        Some(root),
        script.map(|block| block.loc.start as u32).unwrap_or(0),
        template.loc.start as u32,
        &VirtualTsOptions::default(),
    );
    VirtualDocument::from_emission(
        cstr!("{base_uri}.__template.ts").to_string(),
        output.code.to_string(),
        VirtualLanguage::Template,
        output.mapping,
    )
}
