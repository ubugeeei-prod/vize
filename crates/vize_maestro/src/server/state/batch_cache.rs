//! Batch type checker cache and batch-check orchestration.
#![cfg(feature = "native")]

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

use super::ServerState;

/// Batch type check result cache.
pub struct BatchTypeCheckCache {
    /// Diagnostics per file.
    pub diagnostics: DashMap<PathBuf, Vec<vize_canon::BatchDiagnostic>>,
    /// Whether the cache is valid.
    pub valid: std::sync::atomic::AtomicBool,
}

impl BatchTypeCheckCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            diagnostics: DashMap::new(),
            valid: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Invalidate the cache.
    pub fn invalidate(&self) {
        self.valid.store(false, std::sync::atomic::Ordering::SeqCst);
        self.diagnostics.clear();
    }

    /// Check if the cache is valid.
    pub fn is_valid(&self) -> bool {
        self.valid.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Mark the cache as valid.
    pub fn mark_valid(&self) {
        self.valid.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get diagnostics for a file.
    pub fn get_diagnostics(&self, path: &PathBuf) -> Vec<vize_canon::BatchDiagnostic> {
        self.diagnostics
            .get(path)
            .map(|d| d.clone())
            .unwrap_or_default()
    }

    /// Set diagnostics for a file.
    pub fn set_diagnostics(&self, path: PathBuf, diagnostics: Vec<vize_canon::BatchDiagnostic>) {
        self.diagnostics.insert(path, diagnostics);
    }
}

impl Default for BatchTypeCheckCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    /// Get or initialize the batch type checker.
    pub fn get_batch_checker(&self) -> Option<Arc<RwLock<BatchTypeChecker>>> {
        if !self.is_lsp_typecheck_enabled() {
            return None;
        }

        let workspace_root = self.get_workspace_root()?;

        // Try to get existing value first
        if let Some(checker) = self.batch_checker.get() {
            return Some(checker.clone());
        }

        // Try to initialize
        let config = self.get_type_checker_config();
        let corsa_path = config.runtime_path().map(PathBuf::from);
        let options = BatchTypeCheckerOptions {
            tsconfig_path: config.tsconfig.as_ref().map(PathBuf::from),
            virtual_ts_options: vize_canon::virtual_ts::VirtualTsOptions::default(),
        };

        match BatchTypeChecker::with_options_and_corsa_path(
            &workspace_root,
            options,
            corsa_path.as_deref(),
        ) {
            Ok(mut checker) => {
                if self.options_api_enabled() {
                    checker.enable_options_api();
                }
                if self.legacy_vue2_enabled() {
                    checker.enable_legacy_vue2();
                }
                let arc = Arc::new(RwLock::new(checker));
                // get_or_init to handle race condition
                Some(self.batch_checker.get_or_init(|| arc.clone()).clone())
            }
            Err(_) => None,
        }
    }

    /// Check if batch type checker is available.
    pub fn has_batch_checker(&self) -> bool {
        self.is_lsp_typecheck_enabled() && self.batch_checker.get().is_some()
    }

    /// Get the batch type check cache.
    pub fn get_batch_cache(&self) -> &BatchTypeCheckCache {
        &self.batch_cache
    }

    /// Run batch type checking and update the cache.
    pub fn run_batch_type_check(&self) -> Option<vize_canon::BatchTypeCheckResult> {
        if !self.is_lsp_typecheck_enabled() {
            return None;
        }

        let checker = self.get_batch_checker()?;
        let mut checker_guard = checker.write();

        // Scan project if not already scanned
        if checker_guard.file_count() == 0 && checker_guard.scan_project().is_err() {
            return None;
        }

        // Run type check
        let result = checker_guard.check_project().ok()?;

        // Update cache
        self.batch_cache.diagnostics.clear();
        for diag in &result.diagnostics {
            self.batch_cache
                .diagnostics
                .entry(diag.file.clone())
                .or_default()
                .push(diag.clone());
        }
        self.batch_cache.mark_valid();

        Some(result)
    }

    /// Invalidate batch type check cache (e.g., when a file changes).
    pub fn invalidate_batch_cache(&self) {
        self.batch_cache.invalidate();
    }
}
