//! Extraction and freshness-aware caching of external type summaries.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock};
use std::time::SystemTime;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashMap, String, ToCompactString};

use crate::parse_sfc;
use crate::script::build_interface_type_source;
use crate::types::SfcParseOptions;

use super::super::batch_epoch::NO_EPOCH;
use super::super::helpers::is_import_type_only;

/// Type declarations and outgoing type-bearing specifiers extracted from one
/// file on disk.
#[derive(Default)]
pub(super) struct FileTypeSummary {
    pub(super) interfaces: Vec<(String, String)>,
    pub(super) type_aliases: Vec<(String, String)>,
    /// Import/re-export specifiers to follow, in source order.
    pub(super) specifiers: Vec<String>,
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
pub(super) struct CachedFileSummary {
    pub(super) stamp: FileStamp,
    pub(super) validated_epoch: AtomicU64,
    pub(super) summary: FileTypeSummary,
}

impl CachedFileSummary {
    /// Whether this entry may be reused without re-reading the file, paying the
    /// `file_stamp` `metadata` syscall only when the entry has not already been
    /// confirmed this batch. The epoch is stamped forward on a successful
    /// revalidation so later hits in the same batch skip the syscall; outside a
    /// batch (`NO_EPOCH`) every call re-stamps.
    pub(super) fn is_fresh(&self, path: &Path, epoch: u64) -> bool {
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
pub(super) static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, CachedFileSummary>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

pub(super) fn file_stamp(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(metadata) => (metadata.modified().ok(), metadata.len()),
        Err(_) => (None, 0),
    }
}

pub(super) fn build_file_summary(path: &Path) -> Option<FileTypeSummary> {
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

pub(super) fn extract_script_summary(source: &str, summary: &mut FileTypeSummary) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked {
        return;
    }

    extract_script_summary_from_program(&ret.program, source, summary);
}

fn extract_script_summary_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    summary: &mut FileTypeSummary,
) {
    for stmt in program.body.iter() {
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

pub(crate) fn type_import_specifiers_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    is_ts: bool,
) -> Vec<String> {
    if !is_ts {
        return Vec::new();
    }
    let mut summary = FileTypeSummary::default();
    extract_script_summary_from_program(program, source, &mut summary);
    summary.specifiers
}
