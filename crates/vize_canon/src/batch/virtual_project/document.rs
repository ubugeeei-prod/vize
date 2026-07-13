//! Single-document Vue virtual TS generation for editor/socket paths.

use std::path::Path;

use oxc_span::SourceType;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileRequest, SfcCompileSettings, SfcCroquisMode, SfcCroquisSettings,
    SfcDescriptorArtifact, SfcDescriptorProduct, SfcResolvedPropsPolicy, SfcScriptSyntaxProduct,
    SfcScriptSyntaxSnapshot, sfc_source_structure,
};
use vize_atlas::{Compilation, Shared};
use vize_carton::{String as CompactString, ToCompactString};
use vize_croquis::CroquisDocumentProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::{ReliefProduct, TemplateSyntaxMode};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::{ImportRewriter, ImportSourceMap};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping};

use super::build::{
    descriptor_uses_jsx_script, prepend_vue_jsx_reference, virtual_ts_options_for_descriptor,
};
use super::vue_artifact_codegen::{VueArtifactInputs, generate_vue_virtual_ts_from_artifacts};
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions};

/// Rewritten virtual TypeScript for a single in-memory `.vue` document.
#[derive(Clone)]
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
    pub(crate) descriptor: Shared<SfcDescriptorArtifact>,
    pub(crate) script_syntax: Option<Shared<SfcScriptSyntaxSnapshot>>,
}

/// Vue single-document generation options used by editor/socket callers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    let mode = if document_options.legacy_vue2 {
        SfcCroquisMode::LegacyVue2
    } else if document_options.options_api {
        SfcCroquisMode::OptionsApi
    } else {
        SfcCroquisMode::Full
    };
    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation).map_err(graph_error)?;
    let source = compilation
        .add_source(path.to_string_lossy().as_ref(), content)
        .map_err(graph_error)?;
    let filename = path.to_string_lossy().to_compact_string();
    let mut compile_options = SfcCompileOptions::default();
    compile_options.parse.filename = filename.clone();
    let mut compile_settings = SfcCompileSettings::default();
    compile_settings.insert(
        source,
        SfcCompileRequest::new(compile_options, TemplateSyntaxMode::default()),
    );
    compile_settings
        .install(&mut compilation)
        .map_err(graph_error)?;
    let mut croquis_settings = SfcCroquisSettings::new(mode);
    croquis_settings.insert(source, mode);
    croquis_settings.insert_resolved_filename_with_policy(
        source,
        filename,
        SfcResolvedPropsPolicy::PreserveCanonAfterTemplate,
    );
    croquis_settings
        .install(&mut compilation)
        .map_err(graph_error)?;
    let snapshot = compilation.snapshot();
    let mut session = snapshot.query_session();
    let descriptor_artifact = session
        .query::<SfcDescriptorProduct>(source)
        .map_err(graph_error)?
        .shared();
    let descriptor = descriptor_artifact
        .as_result()
        .map_err(|error| CorsaError::SfcParse(error.message.to_compact_string()))?;
    let structure = sfc_source_structure(content);
    let syntax = if structure.has_template {
        Some(
            session
                .query::<ReliefProduct>(source)
                .map_err(graph_error)?,
        )
    } else {
        None
    };
    let semantics = session
        .query::<CroquisDocumentProduct>(source)
        .map_err(graph_error)?;
    let modules = if structure.has_script {
        Some(
            session
                .query::<ModuleSyntaxProduct>(source)
                .map_err(graph_error)?,
        )
    } else {
        None
    };
    let script_syntax = if structure.has_script {
        Some(
            session
                .query::<SfcScriptSyntaxProduct>(source)
                .map_err(graph_error)?
                .shared(),
        )
    } else {
        None
    };
    let effective_options = virtual_ts_options_for_descriptor(options, descriptor);
    let use_tsx_virtual = descriptor_uses_jsx_script(descriptor);
    let source_type = if use_tsx_virtual {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let GeneratedVueFile {
        mut code,
        mut mappings,
        ..
    } = generate_vue_virtual_ts_from_artifacts(
        path,
        content,
        VueArtifactInputs {
            descriptor,
            syntax: syntax.as_ref().and_then(|syntax| syntax.value().as_ref()),
            semantics: semantics.value(),
            modules: modules.as_ref().map(|modules| modules.value()),
            script_syntax: script_syntax.as_deref(),
            extra_template_referenced_names: None,
        },
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
    if use_tsx_virtual {
        prepend_vue_jsx_reference(&mut code, &mut mappings);
    }

    let rewritten = rewriter.rewrite(&code, source_type);
    Ok(VueDocumentVirtualTs {
        code: rewritten.code,
        pre_rewrite_code: code,
        mappings,
        import_source_map: rewritten.source_map,
        source_type,
        virtual_suffix: if use_tsx_virtual { ".tsx" } else { ".ts" },
        descriptor: descriptor_artifact,
        script_syntax,
    })
}

fn graph_error(error: impl std::fmt::Display) -> CorsaError {
    CorsaError::ArtifactGraph(vize_carton::cstr!("{error}"))
}
