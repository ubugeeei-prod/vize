//! Vue virtual-document synchronization for editor Corsa sessions.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, String, cstr};

use super::bridge::CorsaBridge;
use super::types::CorsaBridgeError;
use super::vue_dependencies::{collect_dependency_documents, tsx_vue_import_shim};
use crate::batch::{
    ImportRewriter, ImportSourceMap, VueDocumentVirtualTs, VueDocumentVirtualTsOptions,
};
use crate::file_uri::path_to_file_uri;
use crate::virtual_ts::{VirtualTsOptions, VizeMapping};

/// Options for opening a Vue SFC as a canonical Corsa virtual document.
#[derive(Clone, Copy, Debug, Default)]
pub struct CorsaVueVirtualDocumentOptions {
    pub options_api: bool,
    pub legacy_vue2: bool,
}

/// A Vue SFC projected into the TypeScript document queried by Corsa.
pub struct CorsaVueVirtualDocument {
    pub request_uri: String,
    pub code: String,
    pub pre_rewrite_code: String,
    pub mappings: Vec<VizeMapping>,
    pub import_source_map: ImportSourceMap,
    pub source_type: SourceType,
    pub virtual_suffix: &'static str,
}

pub(crate) struct CorsaVueVirtualProject {
    pub(crate) host: CorsaVueVirtualDocument,
    pub(crate) documents: Vec<(String, String)>,
}

pub(super) struct GeneratedVueDocument {
    pub(super) source_path: PathBuf,
    pub(super) virtual_uri: String,
    pub(super) generated: VueDocumentVirtualTs,
}

impl CorsaBridge {
    /// Generate, sync, and return the canonical `.vue.{ts,tsx}` document used
    /// for editor diagnostics, hover, definition, references, and rename.
    pub async fn open_vue_virtual_document(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        self.open_vue_virtual_document_with_overlays(source_path, content, options, &[])
            .await
    }

    /// Generate and sync a Vue document while preferring unsaved dependency
    /// buffers over their on-disk contents.
    pub async fn open_vue_virtual_document_with_overlays(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, String)],
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        self.open_vue_virtual_document_with_overlays_and_options(
            source_path,
            content,
            options,
            overlays,
            &VirtualTsOptions::default(),
        )
        .await
    }

    /// Generate and sync a Vue document with editor-specific virtual-TS options.
    pub async fn open_vue_virtual_document_with_overlays_and_options(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, String)],
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let overlays = overlays
            .iter()
            .map(|(path, content)| (path.clone(), content.as_str()))
            .collect::<Vec<_>>();
        self.open_vue_virtual_document_with_borrowed_overlays_and_options(
            source_path,
            content,
            options,
            &overlays,
            virtual_ts_options,
        )
        .await
    }

    /// Generate and sync a Vue document without copying unchanged overlay text.
    ///
    /// Only dependency entries reachable from the host's imports are read, so
    /// callers with shared buffer snapshots can lend their text for this call.
    pub async fn open_vue_virtual_document_with_borrowed_overlays_and_options(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let project = build_vue_virtual_project_with_overlays_and_options(
            source_path,
            content,
            options,
            overlays,
            virtual_ts_options,
        )?;
        self.open_virtual_documents_batch(&project.documents)
            .await?;
        Ok(project.host)
    }

    async fn open_virtual_documents_batch(
        &self,
        documents: &[(String, String)],
    ) -> Result<(), CorsaBridgeError> {
        // Owned pairs: the closure runs on the bridge worker thread, so it
        // cannot borrow from this frame (see `super::worker`).
        let owned: Vec<(String, String)> = documents.to_vec();
        let cache_len = self
            .with_client(move |client| {
                let docs: Vec<(&str, &str)> = owned
                    .iter()
                    .map(|(uri, content)| (uri.as_str(), content.as_str()))
                    .collect();
                client
                    .did_open_batch_fast(&docs)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats().set_entries(cache_len as u64);
        Ok(())
    }
}

pub(crate) fn build_vue_virtual_project(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_project_with_overlays(source_path, content, options, &[])
}

pub(crate) fn build_vue_virtual_project_with_overlays(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_project_with_overlays_and_options(
        source_path,
        content,
        options,
        overlays,
        &VirtualTsOptions::default(),
    )
}

fn build_vue_virtual_project_with_overlays_and_options(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
    virtual_ts_options: &VirtualTsOptions,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    let rewriter = ImportRewriter::new();
    let overlays = overlays
        .iter()
        .map(|(path, content)| {
            let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            (key, *content)
        })
        .collect::<FxHashMap<_, _>>();
    // The alias mirror is built before generation, and from the same buffers the
    // dependency walk reads, so a specifier the resolver rewrites always has a
    // materialized target (#3900).
    let alias_context =
        super::vue_dependencies_alias::AliasContext::for_host(source_path, content, &overlays);
    let host = generate_vue_document_with_options(
        source_path,
        content,
        options,
        virtual_ts_options,
        &rewriter,
        Some(&alias_context),
    )?;
    let mut documents = vec![(host.virtual_uri.clone(), host.generated.code.clone())];
    if host.generated.virtual_suffix == ".tsx" {
        documents.push(tsx_vue_import_shim(&host.source_path));
    }
    collect_dependency_documents(
        &mut documents,
        &host,
        options,
        &rewriter,
        &alias_context,
        &overlays,
    );

    let generated = host.generated;
    Ok(CorsaVueVirtualProject {
        host: CorsaVueVirtualDocument {
            request_uri: host.virtual_uri,
            code: generated.code,
            pre_rewrite_code: generated.pre_rewrite_code,
            mappings: generated.mappings,
            import_source_map: generated.import_source_map,
            source_type: generated.source_type,
            virtual_suffix: generated.virtual_suffix,
        },
        documents,
    })
}
/// Generate a Vue document with alias-aware import rewriting: non-relative
/// specifiers the context resolves are pointed at the synced overlay
/// identities through the offset-preserving rewriter (#3900).
pub(super) fn generate_vue_document_with_alias(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    context: &super::vue_dependencies_alias::AliasContext,
) -> Result<GeneratedVueDocument, CorsaBridgeError> {
    generate_vue_document_with_options(
        source_path,
        content,
        options,
        &VirtualTsOptions::default(),
        rewriter,
        Some(context),
    )
}

fn generate_vue_document_with_options(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    virtual_ts_options: &VirtualTsOptions,
    rewriter: &ImportRewriter,
    alias_context: Option<&super::vue_dependencies_alias::AliasContext>,
) -> Result<GeneratedVueDocument, CorsaBridgeError> {
    let source_dir = source_path.parent().map(std::path::Path::to_path_buf);
    let alias_resolver = alias_context.zip(source_dir).map(|(context, dir)| {
        move |specifier: &str| context.resolve_specifier_to_mirror_path(specifier, &dir)
    });
    let generated = crate::batch::virtual_project::generate_vue_document_virtual_ts_with_options_and_alias_resolver(
        source_path,
        content,
        virtual_ts_options,
        rewriter,
        false,
        VueDocumentVirtualTsOptions {
            options_api: options.options_api,
            legacy_vue2: options.legacy_vue2,
        },
        alias_resolver
            .as_ref()
            .map(|resolver| resolver as crate::batch::import_rewriter_alias::AliasSpecifierResolver<'_>),
    )
    .map_err(|error| CorsaBridgeError::CommunicationError(cstr!("{error}")))?;
    let virtual_path = source_path.with_file_name(cstr!(
        "{}{}",
        source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
        generated.virtual_suffix
    ));
    let virtual_uri = path_to_file_uri(&virtual_path);

    Ok(GeneratedVueDocument {
        source_path: source_path.to_path_buf(),
        virtual_uri,
        generated,
    })
}
