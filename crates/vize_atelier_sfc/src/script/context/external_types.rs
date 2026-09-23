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
                    String::from(source.get(type_start..type_end).unwrap_or_default()),
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
                                String::from(source.get(type_start..type_end).unwrap_or_default()),
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
#[expect(
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "tests assert by panicking; insta and fixtures use format!"
)]
mod tests;
