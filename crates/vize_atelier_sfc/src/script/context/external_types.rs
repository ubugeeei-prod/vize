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
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str) {
        if !source.contains("type") || (!source.contains("import") && !source.contains("export")) {
            return;
        }

        let owned_base = canonicalize_or_original(PathBuf::from(filename))
            .unwrap_or_else(|| PathBuf::from(filename));
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return;
        };
        if base_dir.as_os_str().is_empty() {
            return;
        }

        let mut visited = FxHashSet::default();
        // The root source lives in memory (possibly unsaved editor state), so
        // parse it directly; only files read from disk go through the cache.
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root);
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
        return None;
    }

    let base_dir = current_file.parent()?;
    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        base_dir.join(specifier)
    };

    resolve_candidate_path(candidate)
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
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref());
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
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref());
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
        ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref());
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
