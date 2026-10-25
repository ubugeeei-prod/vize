//! Explicit refusal for malformed generic shapes without an honest projection.

use super::{GeneratedVueFile, diagnostic_for_offset, invalid_sfc_fallback_virtual_ts};
use crate::batch::SfcBlockType;
use std::path::Path;
use vize_atelier_sfc::SfcDescriptor;

pub(super) fn refuse(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor<'_>,
) -> GeneratedVueFile {
    let start = crate::virtual_ts::generic_parameters::authored_generic_range(source, descriptor)
        .map_or(0, |range| range.start as u32);
    GeneratedVueFile {
        diagnostic_route: super::super::DiagnosticRoute::ProjectSessionRequired,
        generic_copies: None,
        typed_router_import: false,
        empty_module_exports: false,
        module_exports: None,
        code: invalid_sfc_fallback_virtual_ts(),
        mappings: Vec::new(),
        mapping_meta: Vec::new(),
        semantic_links: Vec::new(),
        prop_default_key_links: Vec::new(),
        diagnostics: vec![diagnostic_for_offset(
            path,
            source,
            start,
            "Cannot safely project this malformed generic attribute".into(),
            SfcBlockType::ScriptSetup,
        )],
    }
}
