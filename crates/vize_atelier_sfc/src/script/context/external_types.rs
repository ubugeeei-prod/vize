use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};
use std::time::SystemTime;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use crate::parse_sfc;
use crate::script::build_interface_type_source;
use crate::types::SfcParseOptions;

use super::ScriptCompileContext;
use super::helpers::is_import_type_only;

/// Type declarations and outgoing type-bearing specifiers extracted from one
/// file on disk.
#[derive(Default)]
struct FileTypeSummary {
    interfaces: Vec<(String, String)>,
    type_aliases: Vec<(String, String)>,
    /// Import/re-export specifiers to follow, in source order.
    specifiers: Vec<String>,
}

/// Freshness stamp for a cached summary: modification time plus file size,
/// so an edit within the same mtime granularity is still detected most of
/// the time.
type FileStamp = (Option<SystemTime>, u64);

/// Process-wide summary cache. Batch compiles and long-lived dev servers walk
/// the same type-barrel closure for every SFC (nuxt-ui re-reads ~200 files per
/// component without this); entries are revalidated against [`FileStamp`] on
/// every use so on-disk edits are picked up.
static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, (FileStamp, FileTypeSummary)>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

fn file_stamp(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(metadata) => (metadata.modified().ok(), metadata.len()),
        Err(_) => (None, 0),
    }
}

fn build_file_summary(path: &Path) -> Option<FileTypeSummary> {
    let content = std::fs::read_to_string(path).ok()?;
    let is_vue = path.extension().is_some_and(|ext| ext == "vue");
    Some(extract_file_summary(&content, is_vue))
}

fn extract_file_summary(content: &str, is_vue: bool) -> FileTypeSummary {
    let mut summary = FileTypeSummary::default();
    if is_vue {
        if let Ok(descriptor) = parse_sfc(content, SfcParseOptions::default()) {
            if let Some(ref script) = descriptor.script {
                extract_script_summary(&script.content, &mut summary);
            }
            if let Some(ref script_setup) = descriptor.script_setup {
                extract_script_summary(&script_setup.content, &mut summary);
            }
        }
    } else {
        extract_script_summary(content, &mut summary);
    }
    summary
}

fn extract_script_summary(source: &str, summary: &mut FileTypeSummary) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked {
        return;
    }

    for stmt in ret.program.body.iter() {
        match stmt {
            Statement::TSInterfaceDeclaration(iface) => {
                summary.interfaces.push((
                    iface.id.name.to_compact_string(),
                    build_interface_type_source(
                        source,
                        iface.id.span.end as usize,
                        iface.body.span.start as usize,
                        iface.body.span.end as usize,
                    ),
                ));
            }
            Statement::TSTypeAliasDeclaration(type_alias) => {
                let type_start = type_alias.type_annotation.span().start as usize;
                let type_end = type_alias.type_annotation.span().end as usize;
                summary.type_aliases.push((
                    type_alias.id.name.to_compact_string(),
                    String::from(&source[type_start..type_end]),
                ));
            }
            Statement::ImportDeclaration(import_decl) => {
                if !import_decl.import_kind.is_type()
                    && !is_import_type_only(import_decl, source)
                    && !import_decl.specifiers.as_ref().is_some_and(|specifiers| {
                        specifiers.iter().any(|specifier| match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                spec.import_kind.is_type()
                            }
                            _ => false,
                        })
                    })
                {
                    continue;
                }
                summary
                    .specifiers
                    .push(import_decl.source.value.to_compact_string());
            }
            // Plain (non-`type`) re-exports forward types as well in TS:
            // `export * from './Link.vue'` in a types barrel re-exports
            // every interface declared there (nuxt-ui resolves LinkProps
            // through exactly this shape). Follow them unconditionally —
            // the `visited` set bounds the traversal and bare specifiers
            // (node_modules) are filtered by import resolution.
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(ref decl) = export_decl.declaration {
                    match decl {
                        oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
                            summary.interfaces.push((
                                iface.id.name.to_compact_string(),
                                build_interface_type_source(
                                    source,
                                    iface.id.span.end as usize,
                                    iface.body.span.start as usize,
                                    iface.body.span.end as usize,
                                ),
                            ));
                        }
                        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                            let type_start = type_alias.type_annotation.span().start as usize;
                            let type_end = type_alias.type_annotation.span().end as usize;
                            summary.type_aliases.push((
                                type_alias.id.name.to_compact_string(),
                                String::from(&source[type_start..type_end]),
                            ));
                        }
                        _ => {}
                    }
                }
                if let Some(ref export_source) = export_decl.source {
                    summary
                        .specifiers
                        .push(export_source.value.to_compact_string());
                }
            }
            Statement::ExportAllDeclaration(export_decl) => {
                summary
                    .specifiers
                    .push(export_decl.source.value.to_compact_string());
            }
            _ => {}
        }
    }
}

const RESOLVE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".js", ".jsx", ".vue",
];
const INDEX_CANDIDATES: &[&str] = &[
    "index.ts",
    "index.tsx",
    "index.d.ts",
    "index.mts",
    "index.cts",
    "index.js",
    "index.jsx",
    "index.vue",
];

impl ScriptCompileContext {
    /// Walk the script's type-bearing imports/re-exports on disk and merge the
    /// interfaces/type aliases they declare into this context.
    ///
    /// `is_ts` must reflect whether the script block is TypeScript
    /// (`lang="ts"`/`"tsx"`, computed once per compile at the call site).
    /// Imported *types* can only be referenced from TypeScript
    /// (`defineProps<Props>()`), so for plain JS the walk would only burn
    /// stat/realpath syscalls — the substring pre-check below misfires on JS
    /// object keys like `type: 'text'` next to any `import`, which is exactly
    /// what the `is_ts` gate cuts off.
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str, is_ts: bool) {
        if !is_ts {
            return;
        }
        if !source.contains("type") || (!source.contains("import") && !source.contains("export")) {
            return;
        }

        // The root source lives in memory (possibly unsaved editor state), so
        // parse it directly; only files read from disk go through the cache.
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root);
        if root.specifiers.is_empty() {
            // Nothing to resolve — skip base-file canonicalization entirely
            // (the common case: scripts with only runtime imports).
            return;
        }

        let owned_base = canonical_base_file(filename);
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return;
        };
        if base_dir.as_os_str().is_empty() {
            return;
        }

        let mut visited = FxHashSet::default();
        for specifier in &root.specifiers {
            self.collect_types_from_specifier(specifier, base_file, &mut visited);
        }
    }

    fn collect_types_from_specifier(
        &mut self,
        specifier: &str,
        current_file: &Path,
        visited: &mut FxHashSet<String>,
    ) {
        let Some(resolved_path) = resolve_import_path(current_file, specifier) else {
            return;
        };

        let key = path_key(&resolved_path);
        if !visited.insert(key) {
            return;
        }

        // Fast path: merge the declarations under the read guard and only
        // clone the (small) specifier list for the recursion below — taking
        // the lock recursively would risk deadlock against writers.
        let stamp = file_stamp(&resolved_path);
        let mut specifiers: Option<std::vec::Vec<String>> = None;
        if let Ok(cache) = FILE_TYPE_CACHE.read()
            && let Some((cached_stamp, summary)) = cache.get(&resolved_path)
            && *cached_stamp == stamp
        {
            self.merge_file_summary(summary);
            specifiers = Some(summary.specifiers.clone());
        }

        let specifiers = match specifiers {
            Some(specifiers) => specifiers,
            None => {
                let Some(summary) = build_file_summary(&resolved_path) else {
                    return;
                };
                self.merge_file_summary(&summary);
                let specifiers = summary.specifiers.clone();
                if let Ok(mut cache) = FILE_TYPE_CACHE.write() {
                    cache.insert(resolved_path.clone(), (stamp, summary));
                }
                specifiers
            }
        };

        for specifier in &specifiers {
            self.collect_types_from_specifier(specifier, &resolved_path, visited);
        }
    }

    fn merge_file_summary(&mut self, summary: &FileTypeSummary) {
        for (name, body) in &summary.interfaces {
            self.interfaces
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
        for (name, body) in &summary.type_aliases {
            self.type_aliases
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
    }
}

/// Base-file canonicalization cache: `(cwd, filename) -> canonical path`.
/// The same SFC filename is canonicalized by several passes per compile
/// (script setup + normal script, croquis prop merge, inline compile) and
/// `canonicalize` walks every path component; a hit is revalidated with a
/// single `is_file` check so a deleted file falls back to a fresh
/// canonicalization, and failures are never cached so files created later
/// are picked up.
static BASE_CANON_CACHE: LazyLock<RwLock<FxHashMap<(PathBuf, String), PathBuf>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

/// Canonicalize the compiled file's own path, falling back to the original
/// path for virtual filenames (in-memory compiles) that don't exist on disk.
fn canonical_base_file(filename: &str) -> PathBuf {
    let path = PathBuf::from(filename);
    // The canonical form of a relative path depends on the process cwd, so
    // the cache key includes it; absolute paths (the batch-compile case) use
    // an empty component and never pay the `getcwd` call.
    let cwd = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    let key = (cwd, filename.to_compact_string());
    if let Ok(cache) = BASE_CANON_CACHE.read()
        && let Some(canonical) = cache.get(&key)
        && canonical.is_file()
    {
        return canonical.clone();
    }

    match path.canonicalize() {
        Ok(canonical) => {
            if let Ok(mut cache) = BASE_CANON_CACHE.write() {
                cache.insert(key, canonical.clone());
            }
            canonical
        }
        Err(_) => path,
    }
}

/// Positive resolution cache: `(importing dir, specifier) -> resolved path`.
/// Resolution probes many extension/index candidates (each a `stat`); a hit
/// is revalidated with a single `is_file` check so deleted files fall back
/// to a full re-resolution, and misses are never cached so newly created
/// files are picked up.
static RESOLVE_CACHE: LazyLock<RwLock<FxHashMap<(PathBuf, String), PathBuf>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

fn resolve_import_path(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    let dir = current_file.parent()?.to_path_buf();
    let key = (dir, specifier.to_compact_string());
    if let Ok(cache) = RESOLVE_CACHE.read()
        && let Some(resolved) = cache.get(&key)
        && resolved.is_file()
    {
        return Some(resolved.clone());
    }

    let resolved = resolve_import_path_uncached(current_file, specifier)?;
    if let Ok(mut cache) = RESOLVE_CACHE.write() {
        cache.insert(key, resolved.clone());
    }
    Some(resolved)
}

fn resolve_import_path_uncached(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if let Some(alias_path) = resolve_at_src_alias(current_file, specifier) {
        return Some(alias_path);
    }

    if !specifier.starts_with('.') && !specifier.starts_with('/') {
        return resolve_bare_specifier(current_file, specifier);
    }

    let base_dir = current_file.parent()?;
    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        base_dir.join(specifier)
    };

    resolve_candidate_path(candidate)
}

/// Resolve a bare specifier (`reka-ui`, `@scope/pkg/sub`) to a package's type
/// declarations through `node_modules`. Only first-party sources step into
/// packages: bare imports *between* packages (every library imports `vue`)
/// would pull huge, mostly `@vue-ignore`d type graphs, so files already
/// inside `node_modules` only follow their relative imports.
fn resolve_bare_specifier(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if specifier.starts_with('#') || specifier.starts_with("node:") {
        return None;
    }
    if current_file
        .components()
        .any(|component| component.as_os_str() == "node_modules")
    {
        return None;
    }

    let (package, subpath) = split_package_specifier(specifier)?;
    for dir in current_file.ancestors().skip(1) {
        let package_dir = dir.join("node_modules").join(&package);
        if package_dir.is_dir() {
            return resolve_package_types(&package_dir, &subpath);
        }
    }
    None
}

/// Split `@scope/name/sub/path` / `name/sub/path` into package name and
/// subpath.
fn split_package_specifier(specifier: &str) -> Option<(String, String)> {
    let segment_count = if specifier.starts_with('@') { 2 } else { 1 };
    let mut split_at = 0;
    let mut seen = 0;
    for (index, byte) in specifier.bytes().enumerate() {
        if byte == b'/' {
            seen += 1;
            if seen == segment_count {
                split_at = index;
                break;
            }
        }
    }
    if seen < segment_count {
        split_at = specifier.len();
    }
    let package = &specifier[..split_at];
    if package.is_empty() {
        return None;
    }
    let subpath = specifier[split_at..].trim_start_matches('/');
    Some((package.to_compact_string(), subpath.to_compact_string()))
}

fn resolve_package_types(package_dir: &Path, subpath: &str) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok();
    let manifest: Option<serde_json::Value> =
        manifest.and_then(|raw| serde_json::from_str(&raw).ok());

    if subpath.is_empty() {
        if let Some(manifest) = &manifest {
            if let Some(types) = manifest
                .get("types")
                .or_else(|| manifest.get("typings"))
                .and_then(|value| value.as_str())
                && let Some(path) = resolve_candidate_path(package_dir.join(types))
            {
                return Some(path);
            }
            if let Some(types) = exports_types_entry(manifest, ".")
                && let Some(path) = resolve_candidate_path(package_dir.join(types))
            {
                return Some(path);
            }
        }
        return resolve_candidate_path(package_dir.join("index.d.ts"));
    }

    if let Some(manifest) = &manifest {
        let mut export_key = String::from("./");
        export_key.push_str(subpath);
        if let Some(types) = exports_types_entry(manifest, export_key.as_str())
            && let Some(path) = resolve_candidate_path(package_dir.join(types))
        {
            return Some(path);
        }
    }
    resolve_candidate_path(package_dir.join(subpath))
}

/// Find the `types` condition for an `exports` entry; conditions may nest
/// (`{ "import": { "types": "./x.d.mts", "default": "./x.mjs" } }`).
fn exports_types_entry(manifest: &serde_json::Value, key: &str) -> Option<String> {
    fn find_types(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::Object(map) => {
                if let Some(types) = map.get("types").and_then(|value| value.as_str()) {
                    return Some(types.to_compact_string());
                }
                map.values().find_map(find_types)
            }
            _ => None,
        }
    }
    find_types(manifest.get("exports")?.get(key)?)
}

fn resolve_at_src_alias(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier.strip_prefix("@/")?;
    let src_dir = current_file
        .parent()?
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "src"))?;

    resolve_candidate_path(src_dir.join(rest))
}

fn resolve_candidate_path(candidate: PathBuf) -> Option<PathBuf> {
    if candidate.is_file() {
        return canonicalize_or_original(candidate);
    }

    if let Some(ts_source_path) = resolve_ts_source_path_for_js_specifier(&candidate) {
        return Some(ts_source_path);
    }

    for ext in RESOLVE_EXTENSIONS {
        let mut with_ext = candidate.clone().into_os_string();
        with_ext.push(ext);
        let path = PathBuf::from(with_ext);
        if path.is_file() {
            return canonicalize_or_original(path);
        }
    }

    if candidate.is_dir() {
        for index_name in INDEX_CANDIDATES {
            let path = candidate.join(index_name);
            if path.is_file() {
                return canonicalize_or_original(path);
            }
        }
    }

    None
}

fn resolve_ts_source_path_for_js_specifier(candidate: &Path) -> Option<PathBuf> {
    let extension = candidate.extension()?.to_str()?;
    let source_extensions: &[&str] = match extension {
        "js" => &["ts", "tsx"],
        "jsx" => &["tsx", "ts"],
        "mjs" => &["mts", "ts"],
        "cjs" => &["cts", "ts"],
        _ => return None,
    };

    for source_extension in source_extensions {
        let source_candidate = candidate.with_extension(source_extension);
        if source_candidate.is_file() {
            return canonicalize_or_original(source_candidate);
        }
    }

    None
}

fn canonicalize_or_original(path: PathBuf) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(canonical) => Some(canonical),
        Err(_) if path.exists() => Some(path),
        Err(_) => None,
    }
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().as_ref().to_compact_string()
}

#[cfg(test)]
mod tests {
    use super::{resolve_at_src_alias, resolve_import_path};
    use std::path::{Path, PathBuf};

    fn temp_project_dir(test_name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vize-sfc-external-types-{}-{}-{}",
            std::process::id(),
            test_name,
            nonce
        ))
    }

    #[test]
    fn resolves_at_alias_from_nearest_src_directory() {
        let project = temp_project_dir("at-alias");
        let components = project.join("packages/frontend/src/components");
        std::fs::create_dir_all(&components).unwrap();
        let target = components.join("Base.vue");
        std::fs::write(&target, "").unwrap();

        let current = components.join("Child.vue");
        let resolved = resolve_at_src_alias(&current, "@/components/Base.vue");
        let target = target.canonicalize().unwrap();

        assert_eq!(resolved.as_deref(), Some(target.as_path()));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn ignores_at_alias_without_src_ancestor() {
        let current = Path::new("/repo/packages/frontend/components/Child.vue");

        assert!(resolve_at_src_alias(current, "@/components/Base.vue").is_none());
    }

    #[test]
    fn leaves_non_at_alias_specifiers_to_existing_resolution() {
        let current = Path::new("/repo/src/components/Child.vue");

        assert!(resolve_import_path(current, "vue").is_none());
    }

    #[test]
    fn resolves_bare_specifier_through_node_modules_types_field() {
        let project = temp_project_dir("bare-types-field");
        let package = project.join("node_modules/some-ui");
        std::fs::create_dir_all(package.join("dist")).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{ "name": "some-ui", "types": "./dist/index.d.ts" }"#,
        )
        .unwrap();
        std::fs::write(
            package.join("dist/index.d.ts"),
            "export interface RootProps { autocomplete?: string }",
        )
        .unwrap();
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();

        let current = components.join("Select.vue");
        let resolved = resolve_import_path(&current, "some-ui");
        let target = package.join("dist/index.d.ts").canonicalize().unwrap();
        assert_eq!(resolved.as_deref(), Some(target.as_path()));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn resolves_scoped_bare_specifier_through_exports_types() {
        let project = temp_project_dir("bare-exports-types");
        let package = project.join("node_modules/@scope/pkg");
        std::fs::create_dir_all(package.join("dist")).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{ "name": "@scope/pkg", "exports": { ".": { "import": { "types": "./dist/main.d.mts", "default": "./dist/main.mjs" } } } }"#,
        )
        .unwrap();
        std::fs::write(package.join("dist/main.d.mts"), "export type T = string").unwrap();
        let src = project.join("src");
        std::fs::create_dir_all(&src).unwrap();

        let current = src.join("App.vue");
        let resolved = resolve_import_path(&current, "@scope/pkg");
        let target = package.join("dist/main.d.mts").canonicalize().unwrap();
        assert_eq!(resolved.as_deref(), Some(target.as_path()));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn does_not_follow_bare_specifiers_from_inside_node_modules() {
        let project = temp_project_dir("bare-from-node-modules");
        let nested = project.join("node_modules/vue");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("package.json"),
            r#"{ "types": "./index.d.ts" }"#,
        )
        .unwrap();
        std::fs::write(nested.join("index.d.ts"), "export type X = 1").unwrap();

        let current = project.join("node_modules/some-ui/dist/index.d.ts");
        assert!(resolve_import_path(&current, "vue").is_none());

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn collects_props_from_node_modules_package_types() {
        let project = temp_project_dir("bare-props-collection");
        let package = project.join("node_modules/some-ui");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{ "name": "some-ui", "types": "./index.d.ts" }"#,
        )
        .unwrap();
        std::fs::write(
            package.join("index.d.ts"),
            "interface RootProps { autocomplete?: string; dir?: string }\nexport { RootProps }",
        )
        .unwrap();
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();

        let current = components.join("Select.vue");
        let source = r#"
import type { RootProps } from "some-ui";

interface SelectProps extends Omit<RootProps, 'dir'> {
  label?: string;
}

const props = defineProps<SelectProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), true);
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("RootProps"));
        assert_eq!(
            ctx.bindings.bindings.get("autocomplete"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(ctx.bindings.bindings.get("dir"), None);

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn resolves_js_type_specifiers_to_ts_sources() {
        let project = temp_project_dir("js-to-ts-type-import");
        let utility = project.join("src/utility");
        let components = project.join("src/components");
        std::fs::create_dir_all(&utility).unwrap();
        std::fs::create_dir_all(&components).unwrap();
        let target = utility.join("paginator.ts");
        std::fs::write(
            &target,
            "export type ExtractorFunction<T> = (item: T) => T;",
        )
        .unwrap();

        let current = components.join("UserList.vue");
        let resolved = resolve_import_path(&current, "@/utility/paginator.js");
        let target = target.canonicalize().unwrap();

        assert_eq!(resolved.as_deref(), Some(target.as_path()));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn collects_type_reexports_from_vue_files() {
        let project = temp_project_dir("vue-type-reexport");
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::write(
            components.join("Base.vue"),
            r#"<script lang="ts">
export interface BaseProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
        )
        .unwrap();
        std::fs::write(
            components.join("index.ts"),
            r#"export { type BaseProps } from "./Base.vue";"#,
        )
        .unwrap();

        let parent = components.join("Parent.vue");
        let source = r#"
import type { BaseProps } from "./index";

interface ParentProps extends BaseProps {}

const props = defineProps<ParentProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("BaseProps"));
        assert_eq!(
            ctx.bindings.bindings.get("as"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("asChild"),
            Some(&crate::types::BindingType::Props)
        );

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn collects_mixed_type_reexports_from_vue_files() {
        let project = temp_project_dir("mixed-vue-type-reexport");
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::write(
            components.join("Content.vue"),
            r#"<script lang="ts">
export interface ContentProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
        )
        .unwrap();
        std::fs::write(
            components.join("index.ts"),
            r#"export {
  default as Content,
  type ContentProps,
} from "./Content.vue";
"#,
        )
        .unwrap();

        let parent = components.join("Parent.vue");
        let source = r#"
import type { ContentProps } from "./index";

interface ParentProps extends ContentProps {}

const props = defineProps<ParentProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("ContentProps"));
        assert_eq!(
            ctx.bindings.bindings.get("as"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("asChild"),
            Some(&crate::types::BindingType::Props)
        );

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn skips_type_import_collection_for_plain_js_scripts() {
        // Regression: the substring pre-check matches plain-JS object keys
        // like `type: 'text'` next to any `import`/`export`, which used to
        // fire the whole stat/realpath resolution walk for every generated
        // JS script. Non-TS blocks must skip collection entirely.
        let project = temp_project_dir("plain-js-gate");
        let components = project.join("src/components");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::write(
            components.join("shared.ts"),
            "export interface InjectedProps { injected?: boolean }",
        )
        .unwrap();

        let current = components.join("Field.vue");
        let source = r#"
import { reactive } from 'vue'
export * from './shared'

const field = reactive({ type: 'text', name: 'email' })
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), false);
        assert!(ctx.interfaces.is_empty());
        assert!(ctx.type_aliases.is_empty());

        // Sanity: the same source *would* pull the interface in for a TS
        // block, so the assertions above genuinely exercise the gate.
        let mut ts_ctx = super::ScriptCompileContext::new(source);
        ts_ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), true);
        assert!(ts_ctx.interfaces.contains_key("InjectedProps"));

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn collects_types_through_plain_star_reexport_barrel() {
        // Regression: a types barrel using plain `export * from './X.vue'`
        // (not `export type *`) still forwards every interface in TS, but the
        // collector skipped non-type re-exports entirely — nuxt-ui's Button
        // lost all `Omit<LinkProps, ...>` props this way.
        let project = temp_project_dir("plain-star-reexport");
        let components = project.join("src/components");
        let types = project.join("src/types");
        std::fs::create_dir_all(&components).unwrap();
        std::fs::create_dir_all(&types).unwrap();
        std::fs::write(
            components.join("Link.vue"),
            r#"<script lang="ts">
export interface LinkProps {
  disabled?: boolean;
  type?: string;
  raw?: boolean;
}
</script>"#,
        )
        .unwrap();
        std::fs::write(
            types.join("index.ts"),
            "export * from '../components/Link.vue'\n",
        )
        .unwrap();

        let parent = components.join("Button.vue");
        let source = r#"
import type { LinkProps } from "../types";

interface ButtonProps extends Omit<LinkProps, 'raw'> {
  label?: string;
}

const props = defineProps<ButtonProps>();
"#;

        let mut ctx = super::ScriptCompileContext::new(source);
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
        ctx.analyze();

        assert!(ctx.interfaces.contains_key("LinkProps"));
        assert_eq!(
            ctx.bindings.bindings.get("disabled"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("type"),
            Some(&crate::types::BindingType::Props)
        );
        assert_eq!(ctx.bindings.bindings.get("raw"), None);

        let _ = std::fs::remove_dir_all(project);
    }
}
