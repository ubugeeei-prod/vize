//! Data transferred from one Canon project snapshot to Corsa consumers.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::String;

use crate::batch::{ImportSourceMap, VueDocumentVirtualTs};
use crate::virtual_ts::{ProjectionMapping, VirtualTsOptions};

/// Options for opening a Vue SFC as a canonical Corsa virtual document.
#[derive(Clone, Copy, Debug, Default)]
pub struct CorsaVueVirtualDocumentOptions {
    pub options_api: bool,
    pub legacy_vue2: bool,
    pub jsx_typecheck: bool,
    pub experimental_patterned_template: bool,
    pub preserve_event_navigation: bool,
    pub dialect: vize_carton::config::VueVersion,
}

/// A Vue SFC projected into the TypeScript document queried by Corsa.
#[derive(Clone)]
pub struct CorsaVueVirtualDocument {
    pub request_uri: String,
    pub code: String,
    pub pre_rewrite_code: String,
    /// Span links and semantic links in pre-rewrite generated TS coordinates.
    pub mapping: ProjectionMapping,
    pub import_source_map: ImportSourceMap,
    pub source_type: SourceType,
    pub virtual_suffix: &'static str,
    pub dependencies: Vec<CorsaVueVirtualDependency>,
    /// Authored files reached from this host, including closed script barrels.
    /// Unlike the shared materialized mirror, this excludes unrelated hosts.
    pub resolved_dependencies: Vec<PathBuf>,
    pub materialized_sources: Vec<CorsaMaterializedSource>,
    pub source_catalog: crate::corsa_bridge::CorsaSourceCatalog,
    /// Private Canon mirror root. Any native URI under this root that is not
    /// retained in the local sources or captured source catalog is unmappable.
    pub session_project_root: Option<PathBuf>,
}

#[derive(Clone)]
pub struct CorsaMaterializedSource {
    pub materialized_path: PathBuf,
    pub source_path: PathBuf,
    pub source: String,
    pub code: String,
    /// Span links and semantic links in pre-rewrite generated TS coordinates.
    pub mapping: ProjectionMapping,
    pub import_source_map: ImportSourceMap,
    pub mapping_kind: CorsaMaterializedMappingKind,
}

/// How coordinates in one Canon-owned materialized file relate to authored
/// source. Only synthetic companions are intentionally unmappable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorsaMaterializedMappingKind {
    Generated,
    AuthoredIdentity,
    Synthetic,
}

impl CorsaMaterializedMappingKind {
    pub fn is_mappable(self) -> bool {
        self != Self::Synthetic
    }
}

#[derive(Clone)]
pub struct CorsaVueVirtualDependency {
    pub source_path: PathBuf,
    pub source: String,
    pub request_uri: String,
    pub code: String,
    /// Span links and semantic links in pre-rewrite generated TS coordinates.
    pub mapping: ProjectionMapping,
    pub import_source_map: ImportSourceMap,
    pub source_type: SourceType,
    pub virtual_suffix: &'static str,
}

/// Native inputs share immutable URI/code payloads without carrying source
/// mappings or a catalog into the native worker.
#[expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes cross native workers"
)]
#[derive(Clone, Default)]
pub(crate) struct CorsaNativeDocumentBatch {
    shared: std::sync::Arc<Vec<(String, String)>>,
    additions: Vec<std::sync::Arc<(String, String)>>,
}

#[expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes cross native workers"
)]
impl CorsaNativeDocumentBatch {
    pub(crate) fn shared(shared: std::sync::Arc<Vec<(String, String)>>) -> Self {
        Self {
            shared,
            additions: Vec::new(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(String, String)> {
        self.shared
            .iter()
            .chain(self.additions.iter().map(std::sync::Arc::as_ref))
    }

    pub(crate) fn len(&self) -> usize {
        self.shared.len() + self.additions.len()
    }

    pub(crate) fn push_shared(&mut self, document: std::sync::Arc<(String, String)>) {
        self.additions.push(document);
    }

    #[cfg(test)]
    pub(crate) fn push(&mut self, document: (String, String)) {
        self.push_shared(std::sync::Arc::new(document));
    }

    #[cfg(test)]
    pub(crate) fn shared_additions_for_test(&self) -> &[std::sync::Arc<(String, String)>] {
        &self.additions
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    pub(crate) fn payload_owner_for_test(&self) -> std::sync::Weak<Vec<(String, String)>> {
        std::sync::Arc::downgrade(&self.shared)
    }

    #[cfg(test)]
    pub(crate) fn to_vec(&self) -> Vec<(String, String)> {
        self.iter().cloned().collect()
    }
}

#[expect(
    clippy::disallowed_types,
    reason = "owned payload bytes cross native workers"
)]
impl From<Vec<(String, String)>> for CorsaNativeDocumentBatch {
    fn from(documents: Vec<(String, String)>) -> Self {
        Self::shared(std::sync::Arc::new(documents))
    }
}

impl PartialEq for CorsaNativeDocumentBatch {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl Eq for CorsaNativeDocumentBatch {}
impl std::fmt::Debug for CorsaNativeDocumentBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
#[cfg(test)]
impl PartialEq<Vec<(String, String)>> for CorsaNativeDocumentBatch {
    fn eq(&self, other: &Vec<(String, String)>) -> bool {
        self.iter().eq(other.iter())
    }
}
#[cfg(test)]
impl PartialEq<CorsaNativeDocumentBatch> for Vec<(String, String)> {
    fn eq(&self, other: &CorsaNativeDocumentBatch) -> bool {
        self.iter().eq(other.iter())
    }
}

pub(crate) struct CorsaVueVirtualProject {
    pub(crate) host: CorsaVueVirtualDocument,
    pub(crate) documents: CorsaNativeDocumentBatch,
    pub(crate) session_project_root: Option<PathBuf>,
    pub(crate) session_config_path: Option<PathBuf>,
    pub(crate) materialized_changes: crate::batch::virtual_project::MaterializedFileDelta,
}

pub(in crate::corsa_bridge) struct GeneratedVueDocument {
    pub(in crate::corsa_bridge) source_path: PathBuf,
    pub(in crate::corsa_bridge) virtual_uri: String,
    pub(in crate::corsa_bridge) generated: VueDocumentVirtualTs,
}

#[derive(Clone, Copy)]
pub(crate) struct CorsaProjectEnvironment<'a> {
    pub(crate) virtual_ts_options: &'a VirtualTsOptions,
    pub(crate) package_routes: &'a crate::PackageRouteResolver,
    pub(crate) project_root: Option<&'a Path>,
    pub(crate) tsconfig_path: Option<&'a Path>,
    pub(crate) editor_session: &'a crate::corsa_bridge::EditorMirrorSession,
}
