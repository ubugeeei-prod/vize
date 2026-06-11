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
use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement, TSInterfaceDeclaration, TSTypeAliasDeclaration};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use serde::Deserialize;
use vize_carton::{
    CompactString, FxHashMap, String, ToCompactString, profile, profiler::CacheStats,
};

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
        if let Some(ref base_url) = self.base_url
            && let Ok(resolved) = self.resolve_from_base(specifier, base_url)
        {
            return Ok(resolved);
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
    /// Parses the module with OXC and collects exported `interface` and
    /// `type` alias declarations — including locally declared types that are
    /// surfaced through `export { ... }` / `export type { ... }` specifiers —
    /// so they can be used for type resolution in defineProps/defineEmits.
    ///
    /// The parse is lazy: it only runs when a caller actually requests
    /// cross-file type definitions for a resolved module, so sources without
    /// type imports never pay for it. The resolved module's content is not
    /// parsed anywhere else, so this is its first and only parse.
    pub fn extract_type_definitions(
        &self,
        content: &str,
    ) -> FxHashMap<CompactString, CompactString> {
        let mut definitions = FxHashMap::default();

        let allocator = Allocator::default();
        let source_type = SourceType::from_path("module.ts").unwrap_or_default();
        let ret = profile!(
            "croquis.import_resolver.oxc_parse",
            Parser::new(&allocator, content, source_type).parse()
        );
        if ret.panicked {
            return definitions;
        }

        // Locally declared (non-exported) types, kept so export specifiers
        // (`export { Name }`, possibly renamed) can surface them afterwards.
        let mut local_definitions: FxHashMap<CompactString, CompactString> = FxHashMap::default();
        // `(exported name, local name)` pairs from export specifiers.
        let mut export_specifiers: Vec<(CompactString, CompactString)> = Vec::new();

        for stmt in &ret.program.body {
            match stmt {
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(decl) = &export.declaration {
                        record_type_declaration(decl, content, &mut definitions);
                    }
                    for spec in &export.specifiers {
                        export_specifiers.push((
                            CompactString::new(spec.exported.name().as_str()),
                            CompactString::new(spec.local.name().as_str()),
                        ));
                    }
                }
                Statement::TSTypeAliasDeclaration(alias) => {
                    record_type_alias(alias, content, &mut local_definitions);
                }
                Statement::TSInterfaceDeclaration(interface) => {
                    record_interface(interface, content, &mut local_definitions);
                }
                _ => {}
            }
        }

        // Export specifiers may appear before or after the declaration they
        // reference, so resolve them once the whole module body is walked.
        for (exported, local) in export_specifiers {
            if let Some(body) = local_definitions.get(&local) {
                definitions.insert(exported, body.clone());
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

/// Record an exported `interface` / `type` alias declaration.
fn record_type_declaration(
    decl: &Declaration<'_>,
    content: &str,
    definitions: &mut FxHashMap<CompactString, CompactString>,
) {
    match decl {
        Declaration::TSTypeAliasDeclaration(alias) => {
            record_type_alias(alias, content, definitions);
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            record_interface(interface, content, definitions);
        }
        _ => {}
    }
}

/// Record a `type Name = ...` alias as `Name -> RHS source text`.
fn record_type_alias(
    alias: &TSTypeAliasDeclaration<'_>,
    content: &str,
    definitions: &mut FxHashMap<CompactString, CompactString>,
) {
    definitions.insert(
        CompactString::new(alias.id.name.as_str()),
        CompactString::new(alias.type_annotation.span().source_text(content).trim()),
    );
}

/// Record an `interface Name { ... }` declaration as `Name -> { body } source text`.
fn record_interface(
    interface: &TSInterfaceDeclaration<'_>,
    content: &str,
    definitions: &mut FxHashMap<CompactString, CompactString>,
) {
    definitions.insert(
        CompactString::new(interface.id.name.as_str()),
        CompactString::new(interface.body.span.source_text(content)),
    );
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
    fn test_extract_ignores_strings_and_comments() {
        let resolver = ImportResolver::default();
        let content = r#"
            // export interface FakeComment { a: string }
            /* export interface FakeBlock { b: number }
               export type FakeBlockAlias = string; */
            const snippet = "export interface FakeString { c: boolean }";
            export const tpl = `export type FakeTemplate = number;`;
            export interface Real { value: number }
        "#;

        let defs = resolver.extract_type_definitions(content);
        assert!(defs.contains_key("Real"));
        assert!(!defs.contains_key("FakeComment"));
        assert!(!defs.contains_key("FakeBlock"));
        assert!(!defs.contains_key("FakeBlockAlias"));
        assert!(!defs.contains_key("FakeString"));
        assert!(!defs.contains_key("FakeTemplate"));
    }

    #[test]
    fn test_extract_generic_interface() {
        let resolver = ImportResolver::default();
        // Nested generics (`Record<string, unknown>`) defeat naive `<[^>]*>`
        // scanning because the first `>` is not the end of the param list.
        let content = r#"
            export interface Box<T extends Record<string, unknown>> {
                value: T;
                items: Array<T>;
            }
        "#;

        let defs = resolver.extract_type_definitions(content);
        let body = defs.get("Box").expect("generic interface extracted");
        assert!(body.contains("value: T"));
        assert!(body.contains("items: Array<T>"));
    }

    #[test]
    fn test_extract_multi_line_nested_interface() {
        let resolver = ImportResolver::default();
        // Two levels of brace nesting break one-level-deep text scanning.
        let content = r#"
            export interface Outer {
                nested: {
                    deep: { a: string };
                };
                sibling: number;
            }
        "#;

        let defs = resolver.extract_type_definitions(content);
        let body = defs.get("Outer").expect("nested interface extracted");
        assert!(body.contains("deep: { a: string }"));
        assert!(body.contains("sibling: number"));
    }

    #[test]
    fn test_extract_union_and_intersection_aliases() {
        let resolver = ImportResolver::default();
        let content = r#"
            export type Status =
                | 'active'
                | 'inactive'
                | 'pending';
            export type Mixed = { base: string } & { extra: boolean };
            export type Handlers = { (e: 'click'): void; (e: 'change', v: string): void };
        "#;

        let defs = resolver.extract_type_definitions(content);

        let status = defs.get("Status").expect("union alias extracted");
        assert!(status.contains("'active'"));
        assert!(status.contains("'pending'"));

        let mixed = defs.get("Mixed").expect("intersection alias extracted");
        assert!(mixed.contains("{ base: string }"));
        assert!(mixed.contains('&'));
        assert!(mixed.contains("{ extra: boolean }"));

        // Semicolon-terminated scanning truncated the body at the first `;`
        // inside the object type; the AST keeps both call signatures.
        let handlers = defs.get("Handlers").expect("object alias extracted");
        assert!(handlers.contains("(e: 'click'): void"));
        assert!(handlers.contains("(e: 'change', v: string): void"));
    }

    #[test]
    fn test_extract_reexported_local_types() {
        let resolver = ImportResolver::default();
        let content = r#"
            export type { Emits as PublicEmits };
            interface Props { msg: string }
            type Emits = (e: 'click') => void;
            export { Props };
        "#;

        let defs = resolver.extract_type_definitions(content);
        assert!(defs.contains_key("Props"));
        assert!(defs.contains_key("PublicEmits"));
        assert!(!defs.contains_key("Emits"));
    }

    #[test]
    fn test_extract_skips_non_exported_types() {
        let resolver = ImportResolver::default();
        let content = r#"
            interface Hidden { a: string }
            type AlsoHidden = number;
            export interface Visible { b: string }
        "#;

        let defs = resolver.extract_type_definitions(content);
        assert!(defs.contains_key("Visible"));
        assert!(!defs.contains_key("Hidden"));
        assert!(!defs.contains_key("AlsoHidden"));
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
