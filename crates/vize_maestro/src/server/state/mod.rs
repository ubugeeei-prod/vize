//! Server state management.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod config;
mod features;
mod virtual_docs;

#[cfg(feature = "native")]
mod batch_cache;
#[cfg(feature = "native")]
mod corsa;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_carton::config::{LinterConfig, TypeCheckerConfig};
use vize_carton::dialect::VueDialect;

#[cfg(feature = "native")]
use std::sync::Arc;
#[cfg(feature = "native")]
use std::sync::OnceLock;

#[cfg(feature = "native")]
use futures::lock::Mutex as AsyncMutex;

#[cfg(feature = "native")]
use vize_canon::{BatchTypeChecker, CorsaBridge};

use crate::document::DocumentStore;
use crate::virtual_code::{VirtualCodeGenerator, VirtualDocuments};

pub use features::LspFeatureConfig;

#[cfg(feature = "native")]
pub use batch_cache::BatchTypeCheckCache;

/// Server state containing all runtime data.
pub struct ServerState {
    /// Document store for managing open documents
    pub documents: DocumentStore,
    /// Virtual code generator (reusable)
    virtual_gen: RwLock<VirtualCodeGenerator>,
    /// Cached virtual documents per file
    virtual_docs_cache: DashMap<Url, VirtualDocuments>,
    /// Parsed metadata for imported components, keyed by resolved path.
    /// Lets template completion skip re-reading + re-parsing + re-analyzing an
    /// imported component on every keystroke; entries are invalidated by the
    /// component file's length + modification time.
    component_metadata_cache:
        DashMap<PathBuf, crate::ide::completion::template::CachedComponentMetadata>,
    /// Enabled LSP feature surface.
    lsp_features: RwLock<LspFeatureConfig>,
    /// Fast path for checking whether type-aware features are enabled.
    lsp_typecheck_enabled: AtomicBool,
    /// Type checker options shared by LSP diagnostics.
    type_checker_config: RwLock<TypeCheckerConfig>,
    /// Vue 3 Options API binding-resolution opt-in from config.
    type_checker_options_api: RwLock<bool>,
    /// Vue 2.7 / Nuxt 2 type checker compatibility flag from config.
    type_checker_legacy_vue2: RwLock<bool>,
    /// Linter options shared by LSP diagnostics.
    linter_config: RwLock<LinterConfig>,
    /// Explicit Vue dialect from config (`dialect` key). `None` means the
    /// dialect is detected structurally per document.
    dialect_config: RwLock<Option<VueDialect>>,
    /// Formatting options (loaded from vize.config.json)
    #[cfg(feature = "glyph")]
    format_options: RwLock<vize_glyph::FormatOptions>,
    /// Corsa bridge for native TypeScript language features (lazy initialized)
    #[cfg(feature = "native")]
    corsa_bridge: OnceLock<Arc<CorsaBridge>>,
    /// Serializes Corsa bridge initialization without tying us to a runtime.
    #[cfg(feature = "native")]
    corsa_init_lock: AsyncMutex<()>,
    /// Flag to track if Corsa initialization has been attempted and failed
    #[cfg(feature = "native")]
    corsa_init_failed: std::sync::atomic::AtomicBool,
    /// Human-readable reason recorded on Corsa init failure, used by
    /// `corsa_init_failure()` to surface diagnostic context to handlers and
    /// tests. Populated alongside `corsa_init_failed` (see #751).
    #[cfg(feature = "native")]
    corsa_init_failure_reason: RwLock<Option<Arc<str>>>,
    /// True once the LSP server has shown the user a one-shot
    /// `window/showMessage` explaining that type checking is unavailable.
    /// Prevents the message from firing once per file.
    #[cfg(feature = "native")]
    typecheck_unavailable_notified: std::sync::atomic::AtomicBool,
    /// Workspace root path
    #[cfg(feature = "native")]
    workspace_root: RwLock<Option<PathBuf>>,
    /// Batch type checker (lazy initialized, sync)
    #[cfg(feature = "native")]
    batch_checker: OnceLock<Arc<RwLock<BatchTypeChecker>>>,
    /// Batch type check result cache
    #[cfg(feature = "native")]
    batch_cache: BatchTypeCheckCache,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    /// Create a new server state.
    pub fn new() -> Self {
        let default_features = LspFeatureConfig::default();
        Self {
            documents: DocumentStore::new(),
            virtual_gen: RwLock::new(VirtualCodeGenerator::new()),
            virtual_docs_cache: DashMap::new(),
            component_metadata_cache: DashMap::new(),
            lsp_features: RwLock::new(default_features),
            lsp_typecheck_enabled: AtomicBool::new(default_features.typecheck),
            type_checker_config: RwLock::new(TypeCheckerConfig::default()),
            type_checker_options_api: RwLock::new(false),
            type_checker_legacy_vue2: RwLock::new(false),
            linter_config: RwLock::new(LinterConfig::default()),
            dialect_config: RwLock::new(None),
            #[cfg(feature = "glyph")]
            format_options: RwLock::new(vize_glyph::FormatOptions::default()),
            #[cfg(feature = "native")]
            corsa_bridge: OnceLock::new(),
            #[cfg(feature = "native")]
            corsa_init_lock: AsyncMutex::new(()),
            #[cfg(feature = "native")]
            corsa_init_failed: std::sync::atomic::AtomicBool::new(false),
            #[cfg(feature = "native")]
            corsa_init_failure_reason: RwLock::new(None),
            #[cfg(feature = "native")]
            typecheck_unavailable_notified: std::sync::atomic::AtomicBool::new(false),
            #[cfg(feature = "native")]
            workspace_root: RwLock::new(None),
            #[cfg(feature = "native")]
            batch_checker: OnceLock::new(),
            #[cfg(feature = "native")]
            batch_cache: BatchTypeCheckCache::new(),
        }
    }

    /// Set the workspace root path.
    #[cfg(feature = "native")]
    pub fn set_workspace_root(&self, path: PathBuf) {
        *self.workspace_root.write() = Some(path);
        // Invalidate batch cache when workspace changes
        self.batch_cache.invalidate();
    }

    /// Check whether LSP type checking is enabled.
    #[inline]
    pub fn is_lsp_typecheck_enabled(&self) -> bool {
        self.lsp_typecheck_enabled.load(Ordering::SeqCst)
    }

    /// Effective Vue dialect for a document, decided once per document version.
    ///
    /// Non-HTML documents (SFCs, scripts) always use the standard Vue dialect.
    /// For standalone HTML documents an explicit `dialect` config key wins;
    /// otherwise the structural petite-vue detection memoized on the open
    /// document is used. `content` is only consulted as a fallback when the
    /// document is not in the store.
    pub fn document_dialect(&self, uri: &Url, content: &str) -> VueDialect {
        if !crate::utils::is_standalone_html_path(uri.path()) {
            return VueDialect::Vue;
        }
        if let Some(configured) = *self.dialect_config.read() {
            return configured;
        }
        match self.documents.get(uri) {
            Some(document) if document.petite_vue_detected() => VueDialect::PetiteVue,
            Some(_) => VueDialect::Vue,
            None => vize_carton::dialect::standalone_html_dialect(None, content),
        }
    }

    /// Get the enabled LSP feature set.
    #[inline]
    pub(crate) fn lsp_features(&self) -> LspFeatureConfig {
        *self.lsp_features.read()
    }

    #[inline]
    pub(crate) fn legacy_vue2_enabled(&self) -> bool {
        *self.type_checker_legacy_vue2.read() || self.lsp_features().legacy_vue2
    }

    /// Resolve Vue 3 Options API template bindings. Implied by legacy mode.
    #[inline]
    pub(crate) fn options_api_enabled(&self) -> bool {
        *self.type_checker_options_api.read()
            || self.lsp_features().options_api
            || self.legacy_vue2_enabled()
    }

    /// Check whether LSP lint diagnostics are enabled.
    #[inline]
    pub fn is_lsp_lint_enabled(&self) -> bool {
        self.lsp_features().lint
    }

    /// Get the workspace root path.
    #[cfg(feature = "native")]
    pub fn get_workspace_root(&self) -> Option<PathBuf> {
        self.workspace_root.read().clone()
    }
}
