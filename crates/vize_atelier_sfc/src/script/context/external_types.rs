use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
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
use super::batch_epoch::{NO_EPOCH, current_batch_epoch};
use super::helpers::is_import_type_only;

mod resolution;
use resolution::{canonical_base_file, path_key, resolve_import_path};

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

/// One cached file summary plus the metadata needed to revalidate it.
///
/// `validated_epoch` records the active batch epoch in which the entry's
/// [`FileStamp`] was last confirmed against disk. It is an atomic so a read
/// hit can stamp it forward under the shared read guard, without upgrading to
/// a write lock.
struct CachedFileSummary {
    stamp: FileStamp,
    validated_epoch: AtomicU64,
    summary: FileTypeSummary,
}

impl CachedFileSummary {
    /// Whether this entry may be reused without re-reading the file, paying the
    /// `file_stamp` `metadata` syscall only when the entry has not already been
    /// confirmed this batch. The epoch is stamped forward on a successful
    /// revalidation so later hits in the same batch skip the syscall; outside a
    /// batch (`NO_EPOCH`) every call re-stamps.
    fn is_fresh(&self, path: &Path, epoch: u64) -> bool {
        if epoch != NO_EPOCH && self.validated_epoch.load(Ordering::Relaxed) == epoch {
            return true;
        }
        if self.stamp == file_stamp(path) {
            self.validated_epoch.store(epoch, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

/// Process-wide summary cache. Batch compiles and long-lived dev servers walk
/// the same type-barrel closure for every SFC (nuxt-ui re-reads ~200 files per
/// component without this); outside a batch entries are revalidated against
/// [`FileStamp`] on every use so on-disk edits are picked up, and within a
/// batch the first hit revalidates and the rest reuse it.
static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, CachedFileSummary>>> =
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
    let follows_value_imports = path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
    });
    Some(extract_file_summary(
        &content,
        is_vue,
        follows_value_imports,
    ))
}

fn extract_file_summary(
    content: &str,
    is_vue: bool,
    follows_value_imports: bool,
) -> FileTypeSummary {
    let mut summary = FileTypeSummary::default();
    if is_vue {
        if let Ok(descriptor) = parse_sfc(content, SfcParseOptions::default()) {
            if let Some(ref script) = descriptor.script {
                extract_script_summary(&script.content, &mut summary, false);
            }
            if let Some(ref script_setup) = descriptor.script_setup {
                extract_script_summary(&script_setup.content, &mut summary, false);
            }
        }
    } else {
        extract_script_summary(content, &mut summary, follows_value_imports);
    }
    summary
}

fn extract_script_summary(
    source: &str,
    summary: &mut FileTypeSummary,
    follows_value_imports: bool,
) {
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
                let type_import = import_decl.import_kind.is_type()
                    || is_import_type_only(import_decl, source)
                    || import_decl.specifiers.as_ref().is_some_and(|specifiers| {
                        specifiers.iter().any(|specifier| match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                spec.import_kind.is_type()
                            }
                            _ => false,
                        })
                    });
                let declaration_value_import = follows_value_imports
                    && import_decl
                        .specifiers
                        .as_ref()
                        .is_some_and(|specifiers| !specifiers.is_empty());
                if !type_import && !declaration_value_import {
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

impl ScriptCompileContext {
    /// Walk the script's type-bearing imports/re-exports on disk and merge the
    /// interfaces/type aliases they declare into this context.
    ///
    /// `is_ts` must reflect whether the script block is TypeScript
    /// (`lang="ts"`/`"tsx"`, computed once per compile at the call site) — it
    /// is the real signal, derived from the parsed SFC, that replaced the old
    /// `source.contains("type")` substring pre-check. Imported *types* can only
    /// be referenced from TypeScript (`defineProps<Props>()`), so for plain JS
    /// the walk would only burn stat/realpath syscalls; the substring heuristic
    /// misfired on JS object keys like `type: 'text'` next to any `import`,
    /// which is exactly what the `is_ts` gate cuts off.
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str, is_ts: bool) {
        if !is_ts {
            return;
        }

        // The root source lives in memory (possibly unsaved editor state), so
        // parse it directly; only files read from disk go through the cache.
        // The parsed specifier list is the precise "is there anything to
        // follow?" signal — strictly tighter than the old substring guard, so
        // no separate text pre-check is needed.
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root, false);
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
        //
        // Within a batch (epoch != NO_EPOCH), an entry already revalidated this
        // epoch is trusted with no syscall: the only `file_stamp` (a `metadata`
        // call) is paid on the first hit of the batch. Outside a batch every
        // hit re-stats, preserving the edit-detection behavior single compiles
        // rely on.
        let epoch = current_batch_epoch();
        let mut specifiers: Option<std::vec::Vec<String>> = None;
        if let Ok(cache) = FILE_TYPE_CACHE.read()
            && let Some(entry) = cache.get(&resolved_path)
            && entry.is_fresh(&resolved_path, epoch)
        {
            self.merge_file_summary(&entry.summary);
            specifiers = Some(entry.summary.specifiers.clone());
        }

        let specifiers = match specifiers {
            Some(specifiers) => specifiers,
            None => {
                // Capture the stamp from the same snapshot we parse so the
                // entry is consistent; a concurrent edit just loses the race
                // and re-stamps on the next miss.
                let stamp = file_stamp(&resolved_path);
                let Some(summary) = build_file_summary(&resolved_path) else {
                    return;
                };
                self.merge_file_summary(&summary);
                let specifiers = summary.specifiers.clone();
                if let Ok(mut cache) = FILE_TYPE_CACHE.write() {
                    cache.insert(
                        resolved_path.clone(),
                        CachedFileSummary {
                            stamp,
                            validated_epoch: AtomicU64::new(epoch),
                            summary,
                        },
                    );
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

#[cfg(test)]
mod tests {
    use super::build_file_summary;
    use super::resolution::{resolve_at_src_alias, resolve_import_path};
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
    fn resolves_javascript_specifiers_to_declaration_sources() {
        let project = temp_project_dir("javascript-to-declaration-imports");
        let dist = project.join("dist");
        std::fs::create_dir_all(&dist).unwrap();
        let current = dist.join("index.d.ts");

        for (specifier, declaration) in [
            ("./index4.js", "index4.d.ts"),
            ("./index4.jsx", "index4.d.ts"),
            ("./index4.mjs", "index4.d.mts"),
            ("./index4.cjs", "index4.d.cts"),
        ] {
            std::fs::write(dist.join(&specifier[2..]), "export {};\n").unwrap();
            let target = dist.join(declaration);
            std::fs::write(&target, "export interface PrimitiveProps {}").unwrap();

            let resolved = resolve_import_path(&current, specifier);
            let target = target.canonicalize().unwrap();
            assert_eq!(resolved.as_deref(), Some(target.as_path()), "{specifier}");
        }

        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn follows_plain_value_imports_only_in_declaration_summaries() {
        let project = temp_project_dir("declaration-value-import-scope");
        std::fs::create_dir_all(&project).unwrap();
        let source = "import './side-effect.js'\nimport { PrimitiveProps } from './chunk.js'\nexport { PrimitiveProps }\n";
        let module = project.join("index.ts");
        std::fs::write(&module, source).unwrap();

        for declaration_name in ["index.d.ts", "index.d.mts", "index.d.cts"] {
            let declaration = project.join(declaration_name);
            std::fs::write(&declaration, source).unwrap();
            let declaration_summary = build_file_summary(&declaration).unwrap();
            assert_eq!(
                declaration_summary.specifiers.as_slice(),
                ["./chunk.js"],
                "{declaration_name}"
            );
        }

        let module_summary = build_file_summary(&module).unwrap();
        assert!(
            module_summary.specifiers.is_empty(),
            "ordinary TS runtime imports must not widen the type graph"
        );

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
