use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock};
use std::time::SystemTime;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use crate::parse_sfc;
use crate::script::build_interface_type_source;
use crate::types::SfcParseOptions;

use super::ScriptCompileContext;
use super::batch_epoch::{NO_EPOCH, current_batch_epoch};
use super::helpers::is_import_type_only;

mod required;

#[derive(Default)]
pub(crate) struct ImportedTypes {
    pub imports: Vec<(String, String)>,
    pub dependencies: Vec<String>,
}
pub(super) mod resolution;
mod sources;
use resolution::{
    canonical_base_file, path_key, project::Project, resolve_import_path_with_project,
};

/// Type declarations and outgoing specifiers extracted from a source module.
#[derive(Debug, Default)]
pub(super) struct FileTypeSummary {
    interfaces: Vec<(String, String)>,
    type_aliases: Vec<(String, String)>,
    /// Import/re-export specifiers to follow, in source order.
    specifiers: Vec<String>,
    /// Local type names and their module requests, for checked macro imports.
    imports: Vec<(String, String)>,
}

/// Metadata hints for configuration discovery. Imported module summaries below
/// validate strong source fingerprints instead of treating metadata as identity.
type FileStamp = (Option<SystemTime>, u64);

/// One transient read. Changed bytes and their digest move to the parser together.
struct FileTypeSource {
    text: String,
    fingerprint: [u8; 32],
}

impl FileTypeSource {
    fn read(path: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let fingerprint = Sha256::digest(text.as_bytes()).into();
        Some(Self {
            text: text.into(),
            fingerprint,
        })
    }
}

/// A parsed summary and a strong source digest, without retaining the whole file.
struct CachedFileSummary {
    fingerprint: [u8; 32],
    validated_epoch: AtomicU64,
    summary: FileTypeSummary,
}

impl CachedFileSummary {
    /// A changed read is returned to the parser so it is never read twice.
    fn revalidate(&self, path: &Path, epoch: u64) -> Result<(), Option<FileTypeSource>> {
        if epoch != NO_EPOCH && self.validated_epoch.load(Ordering::Relaxed) == epoch {
            return Ok(());
        }
        let source = FileTypeSource::read(path);
        if source
            .as_ref()
            .is_some_and(|source| source.fingerprint == self.fingerprint)
        {
            self.validated_epoch.store(epoch, Ordering::Relaxed);
            Ok(())
        } else {
            Err(source)
        }
    }
}

enum SummaryLookup {
    Cached(Vec<String>),
    Reload(Option<FileTypeSource>),
    Miss,
}

/// Shared AST summaries, revalidated once per file in each batch. Standalone
/// compiles compare source digests on each request; metadata cannot prove that a
/// same-length edit within filesystem timestamp granularity is unchanged.
static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, CachedFileSummary>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

fn file_stamp(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(metadata) => (metadata.modified().ok(), metadata.len()),
        Err(_) => (None, 0),
    }
}

#[cfg(test)]
fn build_file_summary(path: &Path) -> Option<FileTypeSummary> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(file_summary_from_source(path, &content))
}

fn file_summary_from_source(path: &Path, content: &str) -> FileTypeSummary {
    let is_vue = path.extension().is_some_and(|ext| ext == "vue");
    let follows_value_imports = path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
    });
    extract_file_summary(content, is_vue, follows_value_imports)
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
                if let Some(specifiers) = &import_decl.specifiers {
                    for specifier in specifiers {
                        let local = match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                &specifier.local
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                &specifier.local
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                &specifier.local
                            }
                        };
                        summary.imports.push((
                            local.name.to_compact_string(),
                            import_decl.source.value.to_compact_string(),
                        ));
                    }
                }
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
        let _ = self.try_collect_imported_types_from_path(source, filename, is_ts, None);
    }

    pub(crate) fn try_collect_imported_types_from_path(
        &mut self,
        source: &str,
        filename: &str,
        is_ts: bool,
        selected: Option<&Project>,
    ) -> Result<ImportedTypes, String> {
        if !is_ts {
            return Ok(ImportedTypes::default());
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
            return Ok(ImportedTypes::default());
        }

        let owned_base = canonical_base_file(filename);
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return Ok(ImportedTypes::default());
        };
        if base_dir.as_os_str().is_empty() {
            return Ok(ImportedTypes::default());
        }

        let discovered = if selected.is_none() {
            resolution::project::for_file(base_file)?
        } else {
            None
        };
        let project = selected.or(discovered.as_deref());

        let mut visited = FxHashSet::default();
        let mut imports = root.imports;
        for specifier in &root.specifiers {
            self.collect_types_from_specifier(
                specifier,
                base_file,
                &mut visited,
                project,
                &mut imports,
            );
        }
        let mut dependencies: Vec<String> = visited.into_iter().collect();
        dependencies.extend(
            Project::discovery_inputs(base_file)?
                .iter()
                .map(|path| path_key(path)),
        );
        if let Some(project) = project {
            dependencies.extend(project.inputs().iter().map(|path| path_key(path)));
        }
        dependencies.sort();
        dependencies.dedup();
        Ok(ImportedTypes {
            imports,
            dependencies,
        })
    }

    fn collect_types_from_specifier(
        &mut self,
        specifier: &str,
        current_file: &Path,
        visited: &mut FxHashSet<String>,
        project: Option<&Project>,
        imports: &mut Vec<(String, String)>,
    ) {
        let Some(resolved_path) =
            resolve_import_path_with_project(current_file, specifier, project)
        else {
            return;
        };

        let key = path_key(&resolved_path);
        if !visited.insert(key) {
            return;
        }

        let epoch = current_batch_epoch();
        let specifiers = match self.cached_file_summary(&resolved_path, epoch, imports) {
            SummaryLookup::Cached(specifiers) => specifiers,
            SummaryLookup::Reload(source) => {
                let Some(specifiers) =
                    self.merge_file_source(&resolved_path, source, epoch, imports)
                else {
                    return;
                };
                specifiers
            }
            SummaryLookup::Miss => {
                let source = FileTypeSource::read(&resolved_path);
                let Some(specifiers) =
                    self.merge_file_source(&resolved_path, source, epoch, imports)
                else {
                    return;
                };
                specifiers
            }
        };

        for specifier in &specifiers {
            self.collect_types_from_specifier(specifier, &resolved_path, visited, project, imports);
        }
    }

    fn cached_file_summary(
        &mut self,
        path: &Path,
        epoch: u64,
        imports: &mut Vec<(String, String)>,
    ) -> SummaryLookup {
        let Ok(cache) = FILE_TYPE_CACHE.read() else {
            return SummaryLookup::Miss;
        };
        let Some(entry) = cache.get(path) else {
            return SummaryLookup::Miss;
        };
        match entry.revalidate(path, epoch) {
            Ok(()) => {
                self.merge_file_summary(&entry.summary);
                imports.extend(entry.summary.imports.iter().cloned());
                SummaryLookup::Cached(entry.summary.specifiers.clone())
            }
            Err(source) => SummaryLookup::Reload(source),
        }
    }

    fn merge_file_source(
        &mut self,
        path: &Path,
        source: Option<FileTypeSource>,
        epoch: u64,
        imports: &mut Vec<(String, String)>,
    ) -> Option<Vec<String>> {
        let source = source?;
        let summary = file_summary_from_source(path, &source.text);
        self.merge_file_summary(&summary);
        imports.extend(summary.imports.iter().cloned());
        let specifiers = summary.specifiers.clone();
        if let Ok(mut cache) = FILE_TYPE_CACHE.write() {
            cache.insert(
                path.to_path_buf(),
                CachedFileSummary {
                    fingerprint: source.fingerprint,
                    validated_epoch: AtomicU64::new(epoch),
                    summary,
                },
            );
        }
        Some(specifiers)
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
