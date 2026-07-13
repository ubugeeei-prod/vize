//! Module registry for caching analyzed files.
//!
//! The registry stores analyzed file results and provides efficient lookup
//! and incremental update capabilities.
//!
//! ## Performance Optimizations
//!
//! - Uses `FxHashMap` for O(1) lookup with fast hashing
//! - Uses `CompactString` for filename storage (SSO for short strings)
//! - Lazy file metadata loading to avoid unnecessary I/O
//! - Source hashing for change detection without file I/O

#[path = "registry/registration.rs"]
mod registration;

use crate::rules::cross_file_reactivity::store_detection::StoreFactories;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::Croquis;
use vize_module::ModuleDocument;

/// Parser-independent import data retained by the cross-file registry.
///
/// Production Atlas paths populate this from `ModuleDocument`; direct analyzer
/// compatibility APIs derive the same shape from Croquis scopes.
#[derive(Debug, Clone)]
pub(crate) struct ModuleImportFact {
    pub(crate) source: CompactString,
    pub(crate) is_type_only: bool,
    pub(crate) local_bindings: Vec<CompactString>,
}

/// Unique identifier for a file in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[repr(transparent)]
pub struct FileId(u32);

impl FileId {
    /// Invalid file ID (sentinel value).
    pub const INVALID: Self = Self(u32::MAX);

    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// Entry for an analyzed module in the registry.
#[derive(Debug)]
pub struct ModuleEntry {
    /// Unique file ID.
    pub id: FileId,
    /// Absolute file path.
    pub path: PathBuf,
    /// File name for display.
    pub filename: CompactString,
    /// Last modification time (for cache invalidation).
    pub mtime: Option<SystemTime>,
    /// Analysis result.
    pub analysis: vize_atlas::Shared<Croquis>,
    /// Source code hash for change detection.
    pub source_hash: u64,
    /// Whether this is a Vue SFC.
    pub is_vue_sfc: bool,
    /// Component name (extracted from filename or defineComponent).
    pub component_name: Option<CompactString>,
    /// Identifiers bound to `defineStore(...)` calls in this module.
    ///
    /// Computed from the AST at registration time so that Pinia store usages
    /// can be resolved structurally rather than by `use*Store` naming.
    pub pinia_stores: StoreFactories,
    /// Imports projected from the source's neutral module product.
    pub(crate) module_imports: Vec<ModuleImportFact>,
    /// Complete owned module facts for production cross-file passes.
    pub(crate) module: Option<ModuleDocument>,
}

/// Registry for tracking all analyzed files in a project.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    /// Map from file path to file ID.
    path_to_id: FxHashMap<PathBuf, FileId>,
    /// Map from file ID to module entry.
    entries: FxHashMap<FileId, ModuleEntry>,
    /// Files whose template source renders a `<slot>` outlet.
    slot_outlets: FxHashSet<FileId>,
    /// Next available file ID.
    next_id: u32,
    /// Project root path.
    project_root: Option<PathBuf>,
}

impl ModuleRegistry {
    /// Create a new empty registry.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with a project root.
    pub fn with_project_root(root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: Some(root.into()),
            ..Default::default()
        }
    }

    /// Set the project root.
    pub fn set_project_root(&mut self, root: impl Into<PathBuf>) {
        self.project_root = Some(root.into());
    }

    /// Get the project root.
    #[inline]
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Get a module entry by file ID.
    #[inline]
    pub fn get(&self, id: FileId) -> Option<&ModuleEntry> {
        self.entries.get(&id)
    }

    /// Get a module entry by file path.
    pub fn get_by_path(&self, path: impl AsRef<Path>) -> Option<&ModuleEntry> {
        let path = path.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref root) = self.project_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        self.path_to_id
            .get(&abs_path)
            .and_then(|id| self.entries.get(id))
    }

    /// Get the file ID for a path.
    pub fn get_id(&self, path: impl AsRef<Path>) -> Option<FileId> {
        let path = path.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref root) = self.project_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        self.path_to_id.get(&abs_path).copied()
    }

    /// Check if a file needs re-analysis (based on mtime).
    pub fn needs_update(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        let Some(entry) = self.get_by_path(path) else {
            return true; // Not in registry
        };

        let Some(cached_mtime) = entry.mtime else {
            return true; // No cached mtime
        };

        let Ok(meta) = std::fs::metadata(path) else {
            return true; // Can't read metadata
        };

        let Ok(current_mtime) = meta.modified() else {
            return true; // Can't get mtime
        };

        current_mtime > cached_mtime
    }

    /// Remove a file from the registry.
    pub fn remove(&mut self, path: impl AsRef<Path>) -> Option<ModuleEntry> {
        let path = path.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref root) = self.project_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        if let Some(id) = self.path_to_id.remove(&abs_path) {
            return self.entries.remove(&id);
        }
        None
    }

    /// Clear all entries from the registry.
    pub fn clear(&mut self) {
        self.path_to_id.clear();
        self.entries.clear();
        self.next_id = 0;
    }

    /// Get the number of registered files.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = &ModuleEntry> {
        self.entries.values()
    }

    /// Get all Vue SFC entries.
    pub fn vue_components(&self) -> impl Iterator<Item = &ModuleEntry> {
        self.entries.values().filter(|e| e.is_vue_sfc)
    }

    /// Find entries by component name.
    pub fn find_by_component_name(&self, name: &str) -> Option<&ModuleEntry> {
        self.entries
            .values()
            .find(|e| e.component_name.as_deref() == Some(name))
    }
}

/// Hash source code for change detection.
#[inline]
fn hash_source(source: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    source.hash(&mut hasher);
    hasher.finish()
}

fn source_renders_slot(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut offset = 0;

    while let Some(index) = source[offset..].find("<slot") {
        let end = offset + index + "<slot".len();
        if bytes
            .get(end)
            .is_some_and(|byte| matches!(byte, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'))
        {
            return true;
        }
        offset = end;
    }

    false
}

/// Extract component name from file path.
///
/// For `MyComponent.vue`, returns `Some("MyComponent")`.
fn extract_component_name(path: &Path) -> Option<CompactString> {
    path.file_stem()
        .map(|s| CompactString::new(s.to_string_lossy().as_ref()))
}

#[cfg(test)]
mod tests {
    use super::{ModuleRegistry, extract_component_name, source_renders_slot};
    use std::path::Path;
    use vize_carton::CompactString;
    use vize_croquis::Croquis;

    #[test]
    fn test_registry_basic() {
        let mut registry = ModuleRegistry::new();

        let (id1, is_new) = registry.register("test.vue", "<template></template>", Croquis::new());
        assert!(is_new);

        let (id2, is_new) = registry.register("test.vue", "<template></template>", Croquis::new());
        assert!(!is_new);
        assert_eq!(id1, id2);

        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_component_name_extraction() {
        let path = Path::new("/src/components/MyButton.vue");
        let name = extract_component_name(path);
        assert_eq!(name, Some(CompactString::new("MyButton")));
    }

    #[test]
    fn test_source_renders_slot_detection() {
        assert!(source_renders_slot("<template><slot /></template>"));
        assert!(source_renders_slot("<slot>fallback</slot>"));
        assert!(!source_renders_slot("<template><slotter /></template>"));
        assert!(!source_renders_slot("<template></template>"));
    }

    #[test]
    fn test_register_tracks_define_store_factories() {
        let mut registry = ModuleRegistry::new();
        let (id, _) = registry.register(
            "stores/user.ts",
            "import { defineStore } from 'pinia'\n\
             export const useUserStore = defineStore('user', {})\n\
             export function useNotAStore() { return 1 }",
            Croquis::new(),
        );

        let entry = registry.get(id).expect("entry");
        // The `defineStore` factory is recognized structurally...
        assert!(entry.pinia_stores.contains("useUserStore"));
        // ...while a plainly-declared function is not, even if it is `use*`.
        assert!(!entry.pinia_stores.contains("useNotAStore"));
    }

    #[test]
    fn test_register_ignores_non_define_store_named_store() {
        let mut registry = ModuleRegistry::new();
        let (id, _) = registry.register(
            "stores/fake.ts",
            "const useThingStore = () => ({})",
            Croquis::new(),
        );

        // Coincidental `use*Store` name, but not a `defineStore` result.
        assert!(
            !registry
                .get(id)
                .unwrap()
                .pinia_stores
                .contains("useThingStore")
        );
    }
}
