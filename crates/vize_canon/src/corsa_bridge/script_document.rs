//! Script-host synchronization for editor Corsa sessions.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, String};

use super::bridge::{CorsaBridge, normalize_document_uri};
use super::types::CorsaBridgeError;
use super::vue_dependencies::collect_script_dependency_documents;
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::batch::ImportRewriter;

impl CorsaBridge {
    /// Open a generated TS/JS host together with every reachable Vue virtual
    /// document, using the same graph materialization as an SFC host.
    pub async fn open_script_virtual_document_with_vue_dependencies(
        &self,
        source_path: &Path,
        request_path: &str,
        code: &str,
        source_type: SourceType,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
    ) -> Result<String, CorsaBridgeError> {
        let (request_uri, documents) = build_script_virtual_project(
            source_path,
            request_path,
            code,
            source_type,
            options,
            overlays,
        );
        self.open_virtual_documents_batch(&documents).await?;
        Ok(request_uri)
    }
}

pub(super) fn build_script_virtual_project(
    source_path: &Path,
    request_path: &str,
    code: &str,
    source_type: SourceType,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
) -> (String, Vec<(String, String)>) {
    let rewriter = ImportRewriter::new();
    let overlays = overlays
        .iter()
        .map(|(path, content)| {
            let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            (key, *content)
        })
        .collect::<FxHashMap<_, _>>();
    let alias_context =
        super::vue_dependencies_alias::AliasContext::for_host_cached(source_path, code, &overlays);
    let request_uri = normalize_document_uri(request_path);
    let mut documents = vec![(request_uri.clone(), code.into())];
    collect_script_dependency_documents(
        &mut documents,
        source_path,
        code,
        source_type,
        options,
        &rewriter,
        &alias_context,
        &overlays,
    );
    (request_uri, documents)
}
