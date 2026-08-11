//! Data transferred from one Canon project snapshot to Corsa consumers.

use std::path::PathBuf;

use oxc_span::SourceType;
use vize_carton::String;

use crate::batch::{ImportSourceMap, VueDocumentVirtualTs};
use crate::virtual_ts::VizeMapping;

/// Options for opening a Vue SFC as a canonical Corsa virtual document.
#[derive(Clone, Copy, Debug, Default)]
pub struct CorsaVueVirtualDocumentOptions {
    pub options_api: bool,
    pub legacy_vue2: bool,
    pub preserve_event_navigation: bool,
    pub dialect: vize_carton::config::VueVersion,
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
    pub dependencies: Vec<CorsaVueVirtualDependency>,
    pub materialized_sources: Vec<CorsaMaterializedSource>,
    /// Private Canon mirror root. Any native URI under this root that is not
    /// present in `materialized_sources` must be rejected by consumers.
    pub session_project_root: Option<PathBuf>,
}

pub struct CorsaMaterializedSource {
    pub materialized_path: PathBuf,
    pub source_path: PathBuf,
    pub source: String,
    pub code: String,
    pub mappings: Vec<VizeMapping>,
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

pub struct CorsaVueVirtualDependency {
    pub source_path: PathBuf,
    pub source: String,
    pub request_uri: String,
    pub code: String,
    pub mappings: Vec<VizeMapping>,
    pub import_source_map: ImportSourceMap,
    pub source_type: SourceType,
    pub virtual_suffix: &'static str,
}

pub(crate) struct CorsaVueVirtualProject {
    pub(crate) host: CorsaVueVirtualDocument,
    pub(crate) documents: Vec<(String, String)>,
    pub(crate) session_project_root: Option<PathBuf>,
    pub(crate) materialized_changes: crate::batch::virtual_project::MaterializedFileDelta,
}

pub(in crate::corsa_bridge) struct GeneratedVueDocument {
    pub(in crate::corsa_bridge) source_path: PathBuf,
    pub(in crate::corsa_bridge) virtual_uri: String,
    pub(in crate::corsa_bridge) generated: VueDocumentVirtualTs,
}
