//! Single-document Vue virtual TS generation for editor/socket paths.

use std::path::Path;

use oxc_span::SourceType;
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{String as CompactString, ToCompactString};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::{ImportRewriter, ImportSourceMap};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping};

use super::build::{descriptor_uses_jsx_script, virtual_ts_options_for_descriptor};
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

/// Rewritten virtual TypeScript for a single in-memory `.vue` document.
pub struct VueDocumentVirtualTs {
    /// `.vue.ts` source after `.vue -> .vue.ts` import rewriting.
    pub code: CompactString,
    /// Generated source before import rewriting, used for sibling overlays.
    pub pre_rewrite_code: CompactString,
    /// Byte-range source mappings in pre-rewrite generated TS coordinates.
    pub mappings: Vec<VizeMapping>,
    /// Source map for `.vue -> .vue.ts` import rewrites.
    pub import_source_map: ImportSourceMap,
    /// Source type used for parsing the generated virtual document.
    pub source_type: SourceType,
    /// Suffix appended to the original `.vue` URI/path for socket-mode Corsa.
    pub virtual_suffix: &'static str,
}

/// Vue single-document generation options used by editor/socket callers.
#[derive(Clone, Copy, Debug, Default)]
pub struct VueDocumentVirtualTsOptions {
    pub options_api: bool,
    pub legacy_vue2: bool,
}

/// Generate the rewritten virtual TypeScript for one in-memory `.vue` document.
pub fn generate_vue_document_virtual_ts(
    path: &Path,
    content: &str,
    options: &VirtualTsOptions,
    rewriter: &ImportRewriter,
    hoist_shared_preamble: bool,
) -> CorsaResult<VueDocumentVirtualTs> {
    generate_vue_document_virtual_ts_with_options(
        path,
        content,
        options,
        rewriter,
        hoist_shared_preamble,
        VueDocumentVirtualTsOptions::default(),
    )
}

pub fn generate_vue_document_virtual_ts_with_options(
    path: &Path,
    content: &str,
    options: &VirtualTsOptions,
    rewriter: &ImportRewriter,
    hoist_shared_preamble: bool,
    document_options: VueDocumentVirtualTsOptions,
) -> CorsaResult<VueDocumentVirtualTs> {
    let descriptor = parse_sfc(
        content,
        SfcParseOptions {
            filename: path.to_string_lossy().to_compact_string(),
            ..Default::default()
        },
    )
    .map_err(|error| CorsaError::SfcParse(error.message.to_compact_string()))?;

    let effective_options = virtual_ts_options_for_descriptor(options, &descriptor);
    let use_tsx_virtual = descriptor_uses_jsx_script(&descriptor);
    let source_type = if use_tsx_virtual {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let GeneratedVueFile { code, mappings, .. } = generate_vue_virtual_ts(
        path,
        content,
        &descriptor,
        &effective_options,
        VueCodegenOptions {
            check_options: VirtualTsCheckOptions::default(),
            preserve_unused_diagnostics: false,
            options_api: document_options.options_api,
            legacy_vue2: document_options.legacy_vue2,
            dialect: vize_carton::config::VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            hoist_shared_preamble,
        },
    )?;

    let rewritten = rewriter.rewrite(&code, source_type);
    Ok(VueDocumentVirtualTs {
        code: rewritten.code,
        pre_rewrite_code: code,
        mappings,
        import_source_map: rewritten.source_map,
        source_type,
        virtual_suffix: if use_tsx_virtual { ".tsx" } else { ".ts" },
    })
}
