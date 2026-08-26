//! Server state management.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod art_template_context;
mod config;
mod features;
mod virtual_docs;
mod workspace_folders;

#[cfg(feature = "native")]
mod batch_cache;
#[cfg(feature = "native")]
mod corsa;
#[cfg(feature = "native")]
mod corsa_overlays;
#[cfg(feature = "native")]
mod global_components;
#[cfg(feature = "native")]
mod workspace_vue_files;

#[cfg(test)]
mod config_tests;
#[cfg(all(test, feature = "native"))]
mod corsa_overlays_perf_tests;
#[cfg(all(test, feature = "native"))]
mod corsa_overlays_tests;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use tower_lsp::lsp_types::Url;
use vize_s0::config::{GlobalTypesConfig, LinterConfig, TypeCheckerConfig};
use vize_s0::dialect::VueDialect;

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
    /// Cached virtual documents per file.
    ///
    /// Stored behind an `Arc` so [`Self::get_virtual_docs`] can hand out an
    /// owned snapshot instead of a `DashMap` shard guard: readers on the LSP's
    /// single executor thread stay alive across `.await` points, and a live
    /// shard guard there deadlocks the whole server against the next
    /// `didOpen`/`didChange` write (#3377, same class as #3315/#3373).
    virtual_docs_cache: DashMap<Url, Arc<VirtualDocuments>>,
    pub(super) open_imports: super::importers::OpenImportIndex,
    /// Importer-scoped package routes reused by synchronous IDE requests.
    /// The resolver validates manifest, link, and source inputs before each
    /// cache hit, so session reuse cannot return a stale export target.
    pub(crate) package_route_resolver: Mutex<vize_canon::PackageRouteResolver>,
    /// Parsed metadata for imported components, keyed by resolved path.
    /// Lets template completion skip re-reading + re-parsing + re-analyzing an
    /// imported component on every keystroke; entries are invalidated by the
    /// component file's length + modification time.
    component_metadata_cache:
        DashMap<PathBuf, crate::ide::completion::template::CachedComponentMetadata>,
    /// Closed `.vue` files announced through workspace file-operation events.
    ///
    /// These stay separate from [`Self::documents`]: document features must
    /// only serve editor-open buffers, while workspace symbol search also
    /// needs to follow files created and deleted on disk mid-session.
    #[cfg(feature = "native")]
    workspace_vue_files: DashMap<Url, ()>,
    /// Enabled LSP feature surface.
    lsp_features: RwLock<LspFeatureConfig>,
    /// Fast path for checking whether type-aware features are enabled.
    lsp_typecheck_enabled: AtomicBool,
    /// Type checker options shared by LSP diagnostics.
    type_checker_config: RwLock<TypeCheckerConfig>,
    /// User-declared template globals shared by every virtual TypeScript path.
    global_types: RwLock<GlobalTypesConfig>,
    /// Vue 3 Options API binding-resolution opt-in from config.
    type_checker_options_api: RwLock<bool>,
    /// Vue 2.7 / Nuxt 2 type checker compatibility flag from config.
    type_checker_legacy_vue2: RwLock<bool>,
    type_checker_vue_version: RwLock<vize_s0::config::VueVersion>,
    /// Opt-in type-aware LSP features for `.jsx`/`.tsx` Vue components (#1498).
    /// Default off: a repository may contain React `.tsx` files that must not
    /// be type-checked as Vue JSX. Set via `typeChecker.jsxTypecheck`.
    type_checker_jsx_typecheck: RwLock<bool>,
    /// Linter options shared by LSP diagnostics.
    linter_config: RwLock<LinterConfig>,
    /// Typed per-rule lint options (`linter.ruleOptions`) for configurable
    /// script rules; loaded alongside `linter_config` (#1891).
    linter_rule_options: RwLock<vize_s0::config::LintRuleOptions>,
    /// Explicit Vue dialect from config (`dialect` key). `None` means the
    /// dialect is detected structurally per document.
    dialect_config: RwLock<Option<VueDialect>>,
    /// Per-workspace-folder linter contexts for true multi-root sessions
    /// (#3240). Empty for clients that only send `rootUri`.
    workspace_folder_configs: RwLock<Vec<workspace_folders::WorkspaceFolderConfig>>,
    /// Formatting options (loaded from vize.config.json)
    #[cfg(feature = "glyph")]
    format_options: RwLock<vize_glyph::FormatOptions>,
    /// Corsa bridge for native TypeScript language features. Lazily
    /// initialized; cleared again by [`Self::retire_corsa_bridge`] when the
    /// backend process dies mid-session so the next request can respawn it
    /// (#3240).
    #[cfg(feature = "native")]
    corsa_bridge: RwLock<Option<Arc<CorsaBridge>>>,
    /// Serializes Corsa bridge initialization without tying us to a runtime.
    #[cfg(feature = "native")]
    corsa_init_lock: AsyncMutex<()>,
    /// Per-document diagnostic passes. Watcher refreshes and consecutive
    /// didChange notifications may be polled concurrently by tower-lsp, but
    /// they must not race the same Corsa virtual document.
    #[cfg(feature = "native")]
    diagnostic_locks: DashMap<Url, Arc<AsyncMutex<()>>>,
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
    /// Cached project declarations that augment Vue's global components.
    #[cfg(feature = "native")]
    global_component_references: global_components::GlobalComponentReferences,
    /// Batch type checker (lazy initialized, sync)
    #[cfg(feature = "native")]
    batch_checker: OnceLock<Arc<RwLock<BatchTypeChecker>>>,
    /// Batch type check result cache
    #[cfg(feature = "native")]
    batch_cache: BatchTypeCheckCache,
    /// Unsaved-buffer overlays handed to Corsa, rebuilt incrementally.
    ///
    /// Rebuilding the whole set per pass costs one full copy of every open
    /// document per keystroke; this keeps the unchanged ones and re-reads only
    /// the document whose revision moved (#3442).
    #[cfg(feature = "native")]
    corsa_overlays: corsa_overlays::CorsaOverlayCache,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        let default_features = LspFeatureConfig::default();
        let package_route_resolver = vize_canon::PackageRouteResolver::default();
        Self {
            documents: DocumentStore::new(),
            virtual_gen: RwLock::new(VirtualCodeGenerator::new()),
            virtual_docs_cache: DashMap::new(),
            open_imports: super::importers::OpenImportIndex::with_package_routes(
                package_route_resolver.clone(),
            ),
            package_route_resolver: Mutex::new(package_route_resolver),
            component_metadata_cache: DashMap::new(),
            #[cfg(feature = "native")]
            workspace_vue_files: DashMap::new(),
            lsp_features: RwLock::new(default_features),
            lsp_typecheck_enabled: AtomicBool::new(default_features.typecheck),
            type_checker_config: RwLock::new(TypeCheckerConfig::default()),
            global_types: RwLock::new(GlobalTypesConfig::default()),
            // Options API matches vue-tsc by default; config may opt out.
            type_checker_options_api: RwLock::new(true),
            type_checker_legacy_vue2: RwLock::new(false),
            type_checker_vue_version: RwLock::new(vize_s0::config::VueVersion::default()),
            // JSX/TSX stays off so React sources remain untouched (#1498).
            type_checker_jsx_typecheck: RwLock::new(false),
            linter_config: RwLock::new(LinterConfig::default()),
            linter_rule_options: RwLock::new(vize_s0::config::LintRuleOptions::default()),
            dialect_config: RwLock::new(None),
            workspace_folder_configs: RwLock::new(Vec::new()),
            #[cfg(feature = "glyph")]
            format_options: RwLock::new(vize_glyph::FormatOptions::default()),
            #[cfg(feature = "native")]
            corsa_bridge: RwLock::new(None),
            #[cfg(feature = "native")]
            corsa_init_lock: AsyncMutex::new(()),
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

    /// Set the workspace root path.
    #[cfg(feature = "native")]
    pub fn set_workspace_root(&self, path: PathBuf) {
        *self.workspace_root.write() = Some(path);
        self.package_route_resolver.lock().clear();
        self.global_component_references.invalidate();
        // Invalidate batch cache when workspace changes
        self.batch_cache.invalidate();
        // Overlays shadow files resolved relative to the workspace root, so a
        // new root retargets them even though no document changed.
        self.corsa_overlays.invalidate();
    }

    /// Close a document and release any cached Corsa overlay immediately.
    pub(crate) fn close_document(&self, uri: &Url) {
        self.documents.close(uri);
        #[cfg(feature = "native")]
        {
            self.corsa_overlays.remove(uri);
            self.remove_idle_diagnostic_lock(uri);
        }
    }

    /// Owned per-document lock for a diagnostic pass. Clone the `Arc` before
    /// awaiting so no DashMap guard survives across a suspension point.
    #[cfg(feature = "native")]
    pub(crate) fn diagnostic_lock(&self, uri: &Url) -> Arc<AsyncMutex<()>> {
        self.diagnostic_locks
            .entry(uri.clone())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    #[cfg(feature = "native")]
    fn remove_idle_diagnostic_lock(&self, uri: &Url) {
        if let dashmap::mapref::entry::Entry::Occupied(entry) =
            self.diagnostic_locks.entry(uri.clone())
            && Arc::strong_count(entry.get()) == 1
        {
            entry.remove();
        }
    }

    /// Rename a document while dropping the overlay cached under its old URI.
    pub(crate) fn rename_document(&self, old_uri: &Url, new_uri: Url) -> bool {
        let renamed = self.documents.rename(old_uri, new_uri);
        #[cfg(feature = "native")]
        if renamed {
            self.corsa_overlays.remove(old_uri);
        }
        renamed
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
            None => vize_s0::dialect::standalone_html_dialect(None, content),
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

    /// Whether type-aware LSP features run for `.jsx`/`.tsx` Vue components.
    ///
    /// Gated by `typeChecker.jsxTypecheck` (default off) — the same opt-in
    /// `vize check` uses — so React `.tsx` files are never type-checked as Vue
    /// JSX unless the user explicitly enables it (#1498).
    #[inline]
    pub(crate) fn jsx_typecheck_enabled(&self) -> bool {
        *self.type_checker_jsx_typecheck.read()
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
