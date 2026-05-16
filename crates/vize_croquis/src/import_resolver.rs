//! Import resolution for TypeScript type definitions.
//!
//! Provides module resolution for external type imports used in Vue compiler macros
//! like `defineProps<Props>()` where `Props` is imported from another file.
//!
//! ## Features
//!
//! - **Path Resolution**: Resolves relative and absolute import paths
//! - **tsconfig.json Support**: Respects path mappings from tsconfig.json
//! - **Caching**: High-performance caching with DashMap for concurrent access
//! - **Type-Only Imports**: Handles `import type { X }` statements

use std::fs;
use std::path::{Path, PathBuf};

use dashmap::DashMap;
use serde::Deserialize;
use vize_carton::{cstr, profiler::CacheStats, CompactString, FxHashMap, String, ToCompactString};

/// Resolved module information
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Absolute path to the resolved file
    pub path: PathBuf,
    /// Module content (lazily loaded)
    pub content: Option<String>,
    /// Whether this is a type-only module (e.g., .d.ts)
    pub is_type_only: bool,
}

/// Import resolution error
#[derive(Debug, Clone)]
pub enum ImportResolveError {
    /// Module not found
    NotFound(String),
    /// Invalid specifier
    InvalidSpecifier(String),
    /// File read error
    ReadError(String),
    /// tsconfig.json parse error
    ConfigError(String),
}

impl std::fmt::Display for ImportResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "Module not found: {}", s),
            Self::InvalidSpecifier(s) => write!(f, "Invalid specifier: {}", s),
            Self::ReadError(s) => write!(f, "Read error: {}", s),
            Self::ConfigError(s) => write!(f, "Config error: {}", s),
        }
    }
}

impl std::error::Error for ImportResolveError {}

/// tsconfig.json compiler options (partial)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsConfigCompilerOptions {
    /// Base URL for module resolution
    base_url: Option<String>,
    /// Path mappings
    paths: Option<FxHashMap<String, Vec<String>>>,
    /// Root directory (reserved for future use)
    #[allow(dead_code)]
    root_dir: Option<String>,
}

/// tsconfig.json structure (partial)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsConfig {
    compiler_options: Option<TsConfigCompilerOptions>,
    extends: Option<String>,
}

/// Import resolver for TypeScript modules
///
/// Resolves import specifiers to their actual file paths, supporting:
/// - Relative imports (`./types`, `../shared/types`)
/// - Absolute imports (via tsconfig paths)
/// - Node modules (basic support)
#[derive(Debug)]
pub struct ImportResolver {
    /// Project root directory
    project_root: PathBuf,
    /// Base URL from tsconfig
    base_url: Option<PathBuf>,
    /// Path mappings from tsconfig
    path_mappings: FxHashMap<String, Vec<String>>,
    /// Resolved module cache (thread-safe)
    cache: DashMap<String, Result<ResolvedModule, ImportResolveError>>,
    /// TypeScript file extensions to try
    extensions: Vec<&'static str>,
    /// Cache statistics
    cache_stats: CacheStats,
}

impl ImportResolver {
    /// Create a new import resolver
    ///
    /// # Arguments
    /// * `project_root` - The root directory of the project
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let mut resolver = Self {
            project_root: project_root.clone(),
            base_url: None,
            path_mappings: FxHashMap::default(),
            cache: DashMap::new(),
            extensions: vec![".ts", ".tsx", ".d.ts", ".js", ".jsx"],
            cache_stats: CacheStats::new(),
        };

        // Try to load tsconfig.json
        resolver.load_tsconfig(&project_root);

        resolver
    }

    /// Create a resolver with custom configuration
    pub fn with_config(
        project_root: impl Into<PathBuf>,
        base_url: Option<PathBuf>,
        path_mappings: FxHashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            project_root: project_root.into(),
            base_url,
            path_mappings,
            cache: DashMap::new(),
            extensions: vec![".ts", ".tsx", ".d.ts", ".js", ".jsx"],
            cache_stats: CacheStats::new(),
        }
    }

    /// Load tsconfig.json and extract path mappings
    fn load_tsconfig(&mut self, dir: &Path) {
        let tsconfig_path = dir.join("tsconfig.json");
        if !tsconfig_path.exists() {
            return;
        }

        let content = match fs::read_to_string(&tsconfig_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let config: TsConfig = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(_) => return,
        };

        if let Some(ref compiler_options) = config.compiler_options {
            // Set base URL
            if let Some(ref base) = compiler_options.base_url {
                self.base_url = Some(dir.join(base));
            }

            // Set path mappings
            if let Some(ref paths) = compiler_options.paths {
                self.path_mappings = paths.clone();
            }
        }

        // Handle extends (basic support)
        if let Some(ref extends) = config.extends {
            let extended_path = dir.join(extends);
            if let Some(parent) = extended_path.parent() {
                self.load_tsconfig(parent);
            }
        }
    }

    /// Resolve an import specifier to a module
    ///
    /// # Arguments
    /// * `specifier` - The import specifier (e.g., `./types`, `@/types`)
    /// * `from_file` - The file containing the import statement
    ///
    /// # Returns
    /// The resolved module or an error
    pub fn resolve(
        &self,
        specifier: &str,
        from_file: &Path,
    ) -> Result<ResolvedModule, ImportResolveError> {
        // Create cache key
        #[allow(clippy::disallowed_macros)]
        let cache_key = format!("{}:{specifier}", from_file.display());

        // Check cache first
        if let Some(cached) = self.cache.get(cache_key.as_str()) {
            self.cache_stats.hit();
            return cached.clone();
        }

        self.cache_stats.miss();

        // Resolve the module
        let result = self.resolve_uncached(specifier, from_file);

        // Cache the result
        self.cache.insert(cache_key.into(), result.clone());
        self.cache_stats.set_entries(self.cache.len() as u64);

        result
    }

    /// Resolve without caching
    fn resolve_uncached(
        &self,
        specifier: &str,
        from_file: &Path,
    ) -> Result<ResolvedModule, ImportResolveError> {
        // Skip node_modules for now (future: support type definitions)
        if specifier.starts_with("node:") || !specifier.contains('/') && !specifier.starts_with('.')
        {
            return Err(ImportResolveError::NotFound({
                #[allow(clippy::disallowed_macros)]
                let s = format!("Node module resolution not supported: {specifier}");
                s.into()
            }));
        }

        // Try relative resolution
        if specifier.starts_with('.') {
            return self.resolve_relative(specifier, from_file);
        }

        // Try path mapping resolution
        if let Some(resolved) = self.resolve_with_paths(specifier)? {
            return Ok(resolved);
        }

        // Try base URL resolution
        if let Some(ref base_url) = self.base_url {
            if let Ok(resolved) = self.resolve_from_base(specifier, base_url) {
                return Ok(resolved);
            }
        }

        Err(ImportResolveError::NotFound(specifier.to_compact_string()))
    }

    /// Resolve a relative import
    fn resolve_relative(
        &self,
        specifier: &str,
        from_file: &Path,
    ) -> Result<ResolvedModule, ImportResolveError> {
        let from_dir = from_file
            .parent()
            .ok_or_else(|| ImportResolveError::InvalidSpecifier(specifier.to_compact_string()))?;

        let target = from_dir.join(specifier);
        self.try_resolve_file(&target)
    }

    /// Resolve using path mappings
    fn resolve_with_paths(
        &self,
        specifier: &str,
    ) -> Result<Option<ResolvedModule>, ImportResolveError> {
        for (pattern, replacements) in &self.path_mappings {
            // Handle wildcard patterns (e.g., "@/*" -> ["src/*"])
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 2];
                if let Some(suffix) = specifier.strip_prefix(prefix) {
                    for replacement in replacements {
                        let replacement_prefix = &replacement[..replacement.len() - 1];
                        let base = self.base_url.as_ref().unwrap_or(&self.project_root);
                        #[allow(clippy::disallowed_macros)]
                        let target = base.join(format!("{replacement_prefix}{suffix}"));
                        if let Ok(resolved) = self.try_resolve_file(&target) {
                            return Ok(Some(resolved));
                        }
                    }
                }
            }
            // Exact match
            else if specifier == pattern {
                for replacement in replacements {
                    let base = self.base_url.as_ref().unwrap_or(&self.project_root);
                    let target = base.join(replacement);
                    if let Ok(resolved) = self.try_resolve_file(&target) {
                        return Ok(Some(resolved));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Resolve from base URL
    fn resolve_from_base(
        &self,
        specifier: &str,
        base_url: &Path,
    ) -> Result<ResolvedModule, ImportResolveError> {
        let target = base_url.join(specifier);
        self.try_resolve_file(&target)
    }

    /// Try to resolve a file path with various extensions
    fn try_resolve_file(&self, path: &Path) -> Result<ResolvedModule, ImportResolveError> {
        // Try exact path first
        if path.exists() && path.is_file() {
            return self.create_resolved_module(path);
        }

        // Try with extensions
        for ext in &self.extensions {
            let with_ext = path.with_extension(&ext[1..]); // Remove leading dot
            if with_ext.exists() && with_ext.is_file() {
                return self.create_resolved_module(&with_ext);
            }
        }

        // Try as directory with index file
        if path.exists() && path.is_dir() {
            for ext in &self.extensions {
                #[allow(clippy::disallowed_macros)]
                let index = path.join(format!("index{}", ext));
                if index.exists() && index.is_file() {
                    return self.create_resolved_module(&index);
                }
            }
        }

        // Try path.ts if no extension
        if path.extension().is_none() {
            for ext in &self.extensions {
                #[allow(clippy::disallowed_macros)]
                let with_ext = PathBuf::from(format!("{}{}", path.display(), ext));
                if with_ext.exists() && with_ext.is_file() {
                    return self.create_resolved_module(&with_ext);
                }
            }
        }

        Err(ImportResolveError::NotFound(
            path.display().to_compact_string(),
        ))
    }

    /// Create a resolved module from a path
    fn create_resolved_module(&self, path: &Path) -> Result<ResolvedModule, ImportResolveError> {
        let canonical = path
            .canonicalize()
            .map_err(|e| ImportResolveError::ReadError(e.to_compact_string()))?;

        let is_type_only = canonical
            .extension()
            .map(|ext| ext == "d.ts")
            .unwrap_or(false)
            || canonical
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".d.ts"))
                .unwrap_or(false);

        Ok(ResolvedModule {
            path: canonical,
            content: None, // Lazy loaded
            is_type_only,
        })
    }

    /// Get the content of a resolved module
    pub fn get_content(&self, module: &ResolvedModule) -> Result<String, ImportResolveError> {
        fs::read_to_string(&module.path)
            .map(|s| s.into())
            .map_err(|e| ImportResolveError::ReadError(e.to_compact_string()))
    }

    /// Extract type definitions from a module's content
    ///
    /// Extracts interface and type alias definitions that can be used
    /// for type resolution in defineProps/defineEmits.
    pub fn extract_type_definitions(
        &self,
        content: &str,
    ) -> FxHashMap<CompactString, CompactString> {
        let mut definitions = FxHashMap::default();

        // Simple regex-based extraction for common patterns
        // TODO: Use OXC for more accurate parsing

        // Extract interface definitions
        let interface_re = regex::Regex::new(
            r"(?s)export\s+interface\s+(\w+)(?:<[^>]*>)?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}",
        );
        if let Ok(re) = interface_re {
            for cap in re.captures_iter(content) {
                if let (Some(name), Some(body)) = (cap.get(1), cap.get(2)) {
                    definitions.insert(
                        CompactString::new(name.as_str()),
                        cstr!("{{ {} }}", body.as_str().trim()),
                    );
                }
            }
        }

        // Extract type alias definitions
        let type_re = regex::Regex::new(r"export\s+type\s+(\w+)(?:<[^>]*>)?\s*=\s*([^;]+);");
        if let Ok(re) = type_re {
            for cap in re.captures_iter(content) {
                if let (Some(name), Some(body)) = (cap.get(1), cap.get(2)) {
                    definitions.insert(
                        CompactString::new(name.as_str()),
                        CompactString::new(body.as_str().trim()),
                    );
                }
            }
        }

        definitions
    }

    /// Clear the resolution cache
    pub fn clear_cache(&self) {
        self.cache.clear();
        self.cache_stats.reset();
        self.cache_stats.set_entries(0);
    }

    /// Get cache statistics
    #[inline]
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    /// Get the project root
    #[inline]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the base URL
    #[inline]
    pub fn base_url(&self) -> Option<&Path> {
        self.base_url.as_deref()
    }

    /// Get path mappings
    #[inline]
    pub fn path_mappings(&self) -> &FxHashMap<String, Vec<String>> {
        &self.path_mappings
    }
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::ImportResolver;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_relative_resolution() {
        let dir = tempdir().unwrap();
        let types_file = dir.path().join("types.ts");
        fs::write(&types_file, "export interface Props { msg: string }").unwrap();

        let component_file = dir.path().join("Component.vue");
        fs::write(&component_file, "").unwrap();

        let resolver = ImportResolver::new(dir.path());
        let result = resolver.resolve("./types", &component_file);

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.path, types_file.canonicalize().unwrap());
    }

    #[test]
    fn test_path_mapping_resolution() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        let types_file = src_dir.join("types.ts");
        fs::write(&types_file, "export interface Props { msg: string }").unwrap();

        // Create tsconfig with path mapping
        let tsconfig = dir.path().join("tsconfig.json");
        fs::write(
            &tsconfig,
            r#"{
                "compilerOptions": {
                    "baseUrl": ".",
                    "paths": {
                        "@/*": ["src/*"]
                    }
                }
            }"#,
        )
        .unwrap();

        let component_file = dir.path().join("Component.vue");
        fs::write(&component_file, "").unwrap();

        let resolver = ImportResolver::new(dir.path());
        let result = resolver.resolve("@/types", &component_file);

        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_type_definitions() {
        let resolver = ImportResolver::default();
        let content = r#"
            export interface Props {
                msg: string;
                count?: number;
            }

            export type Emits = {
                (e: 'click'): void;
            }
        "#;

        let defs = resolver.extract_type_definitions(content);
        assert!(defs.contains_key("Props"));
        assert!(defs.contains_key("Emits"));
    }

    #[test]
    fn test_caching() {
        let dir = tempdir().unwrap();
        let types_file = dir.path().join("types.ts");
        fs::write(&types_file, "export interface Props { msg: string }").unwrap();

        let component_file = dir.path().join("Component.vue");
        fs::write(&component_file, "").unwrap();

        let resolver = ImportResolver::new(dir.path());

        // First resolution
        let result1 = resolver.resolve("./types", &component_file);
        assert!(result1.is_ok());

        // Second resolution (should hit cache)
        let result2 = resolver.resolve("./types", &component_file);
        assert!(result2.is_ok());

        // Results should be equivalent
        assert_eq!(result1.unwrap().path, result2.unwrap().path);
    }
}
