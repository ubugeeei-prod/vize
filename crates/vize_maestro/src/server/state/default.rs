//! The existing default constructor, separate from the growing state facade.

use std::sync::atomic::AtomicBool;

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use vize_l0::config::{GlobalTypesConfig, TypeCheckerConfig};

use super::{LspFeatureConfig, ServerState, resident};
use crate::document::DocumentStore;
use crate::virtual_code::VirtualCodeGenerator;

#[cfg(feature = "native")]
use super::{BatchTypeCheckCache, corsa_overlays, global_components, workspace_project_files};
#[cfg(feature = "experimental-source-navigation")]
use super::{module_links, native_names};
#[cfg(feature = "native")]
use futures::lock::Mutex as AsyncMutex;
#[cfg(feature = "native")]
use std::sync::OnceLock;

impl ServerState {
    pub fn new() -> Self {
        let default_features = LspFeatureConfig::default();
        let package_route_resolver = vize_canon::PackageRouteResolver::default();
        Self {
            documents: DocumentStore::new(),
            resident: resident::ResidentCache::default(),
            virtual_gen: RwLock::new(VirtualCodeGenerator::new()),
            virtual_docs_cache: DashMap::new(),
            binding_occurrences: DashMap::new(),
            lint_hover_cache: DashMap::new(),
            open_imports: crate::server::importers::OpenImportIndex::with_package_routes(
                package_route_resolver.clone(),
            ),
            package_route_resolver: Mutex::new(package_route_resolver),
            component_metadata_cache: DashMap::new(),
            #[cfg(feature = "native")]
            workspace_vue_files: DashMap::new(),
            #[cfg(feature = "native")]
            workspace_project_files: workspace_project_files::Inventory::default(),
            lsp_features: RwLock::new(default_features),
            lsp_typecheck_enabled: AtomicBool::new(default_features.typecheck),
            type_checker_config: RwLock::new((TypeCheckerConfig::default(), 60_000)),
            #[cfg(feature = "experimental-source-navigation")]
            module_links: RwLock::new(module_links::Session::default()),
            global_types: RwLock::new(GlobalTypesConfig::default()),
            // Options API matches vue-tsc by default; config may opt out.
            type_checker_options_api: RwLock::new(true),
            type_checker_legacy_vue2: RwLock::new(false),
            type_checker_vue_version: RwLock::new(vize_l0::config::VueVersion::default()),
            // JSX/TSX stays off so React sources remain untouched (#1498).
            type_checker_jsx_typecheck: RwLock::new(false),
            experimental_patterned_template: AtomicBool::new(false),
            #[cfg(feature = "experimental-source-navigation")]
            native_linked_editing: AtomicBool::new(false),
            #[cfg(feature = "experimental-source-navigation")]
            native_names: RwLock::new(native_names::Generations::default()),
            linter_config: RwLock::default(),
            linter_rule_options: RwLock::new(vize_l0::config::ConfigLintRuleOptions::default()),
            dialect_config: RwLock::new(None),
            workspace_folder_configs: RwLock::new(Vec::new()),
            #[cfg(feature = "glyph")]
            format_options: RwLock::new(vize_glyph::FormatOptions::default()),
            #[cfg(feature = "native")]
            corsa_bridge: RwLock::new(None),
            #[cfg(feature = "native")]
            corsa_init_lock: AsyncMutex::new(()),
            #[cfg(feature = "native")]
            corsa_request_lock: AsyncMutex::new(()),
            #[cfg(feature = "native")]
            corsa_environment_revision: std::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "native")]
            corsa_environment_changes: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(feature = "native")]
            diagnostic_locks: DashMap::new(),
            #[cfg(feature = "native")]
            corsa_init_failed: std::sync::atomic::AtomicBool::new(false),
            #[cfg(feature = "native")]
            corsa_init_failure_reason: RwLock::new(None),
            #[cfg(feature = "native")]
            typecheck_unavailable_notified: std::sync::atomic::AtomicBool::new(false),
            #[cfg(feature = "native")]
            workspace_root: RwLock::new(None),
            #[cfg(feature = "native")]
            global_component_references: global_components::GlobalComponentReferences::new(),
            #[cfg(feature = "native")]
            batch_checker: OnceLock::new(),
            #[cfg(feature = "native")]
            batch_cache: BatchTypeCheckCache::new(),
            #[cfg(feature = "native")]
            corsa_overlays: corsa_overlays::CorsaOverlayCache::default(),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
