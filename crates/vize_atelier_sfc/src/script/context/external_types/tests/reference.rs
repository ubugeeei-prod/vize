//! The walk every component ran on its own before module graphs were shared,
//! kept as the reference for the shared walk: it must merge the same
//! declarations, imports and dependencies, and its work is counted with the
//! same steps. The batch generation is a parameter, so a test never opens a
//! process-wide batch.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock};

use vize_carton::{FxHashMap, FxHashSet, String};

use super::super::super::batch_epoch::NO_EPOCH;
use super::super::resolution::{path_key, project::Project, resolve_import_path_with_project};
use super::super::summary_cache::FileTypeSource;
use super::super::walk::{RequestedExports, TypeDependency};
use super::super::work::{Step, count};
use super::super::{FileTypeSummary, ScriptCompileContext, file_summary_from_source};

struct CachedFileSummary {
    fingerprint: [u8; 32],
    validated_epoch: AtomicU64,
    summary: FileTypeSummary,
}

impl CachedFileSummary {
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
    Cached(Vec<TypeDependency>),
    Reload(Option<FileTypeSource>),
    Miss,
}

static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, CachedFileSummary>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

struct TypeWalk<'a> {
    visited: FxHashSet<String>,
    traversed: FxHashSet<String>,
    project: Option<&'a Project>,
    imports: Vec<(String, String)>,
    epoch: u64,
}

/// Merge what `roots` reach into `ctx`; return the walked imports and the
/// sorted dependency names.
pub(super) fn collect(
    ctx: &mut ScriptCompileContext,
    roots: &[TypeDependency],
    base_file: &Path,
    project: Option<&Project>,
    epoch: u64,
) -> (Vec<(String, String)>, Vec<String>) {
    let mut walk = TypeWalk {
        visited: FxHashSet::default(),
        traversed: FxHashSet::default(),
        project,
        imports: Vec::new(),
        epoch,
    };
    for dependency in roots {
        collect_types_from_dependency(
            ctx,
            dependency,
            &RequestedExports::All,
            base_file,
            &mut walk,
        );
    }
    let mut files: Vec<String> = walk.visited.into_iter().collect();
    files.sort();
    count(Step::Dependency, files.len());
    (walk.imports, files)
}

fn collect_types_from_dependency(
    ctx: &mut ScriptCompileContext,
    dependency: &TypeDependency,
    requested: &RequestedExports,
    current_file: &Path,
    walk: &mut TypeWalk<'_>,
) {
    count(Step::Edge, 1);
    if requested.is_empty() {
        return;
    }
    let (specifier, requested) = match dependency {
        TypeDependency::Import { specifier, names } => (
            specifier,
            names.as_ref().map_or(RequestedExports::All, |names| {
                RequestedExports::Names(names.clone())
            }),
        ),
        TypeDependency::NamedReexport {
            specifier,
            bindings,
        } => {
            let names = match requested {
                RequestedExports::All => bindings.iter().map(|(_, local)| local.clone()).collect(),
                RequestedExports::Names(requested) => bindings
                    .iter()
                    .filter(|(exported, _)| requested.contains(exported))
                    .map(|(_, local)| local.clone())
                    .collect::<Vec<_>>(),
            };
            if names.is_empty() {
                return;
            }
            (specifier, RequestedExports::Names(names))
        }
        TypeDependency::StarReexport { specifier } => (specifier, requested.clone()),
    };
    collect_types_from_specifier(ctx, specifier, &requested, current_file, walk);
}

fn collect_types_from_specifier(
    ctx: &mut ScriptCompileContext,
    specifier: &str,
    requested: &RequestedExports,
    current_file: &Path,
    walk: &mut TypeWalk<'_>,
) {
    let Some(resolved_path) =
        resolve_import_path_with_project(current_file, specifier, walk.project, walk.epoch)
    else {
        return;
    };

    let key = path_key(&resolved_path);
    walk.visited.insert(key.clone());
    let traversal_key = vize_carton::cstr!("{key}\0{}", cache_key(requested));
    if !walk.traversed.insert(traversal_key) {
        return;
    }

    let epoch = walk.epoch;
    let dependencies = match cached_file_summary(ctx, &resolved_path, epoch, &mut walk.imports) {
        SummaryLookup::Cached(dependencies) => dependencies,
        SummaryLookup::Reload(source) => {
            let Some(dependencies) =
                merge_file_source(ctx, &resolved_path, source, epoch, &mut walk.imports)
            else {
                return;
            };
            dependencies
        }
        SummaryLookup::Miss => {
            let source = FileTypeSource::read(&resolved_path);
            let Some(dependencies) =
                merge_file_source(ctx, &resolved_path, source, epoch, &mut walk.imports)
            else {
                return;
            };
            dependencies
        }
    };

    let star_requested = requested.without_named_reexports(&dependencies);
    for dependency in &dependencies {
        let requested = if matches!(dependency, TypeDependency::StarReexport { .. }) {
            &star_requested
        } else {
            requested
        };
        collect_types_from_dependency(ctx, dependency, requested, &resolved_path, walk);
    }
}

fn cache_key(requested: &RequestedExports) -> String {
    match requested {
        RequestedExports::All => String::from("*"),
        RequestedExports::Names(names) => {
            let mut names = names.clone();
            names.sort();
            names.dedup();
            String::from(names.join("\0"))
        }
    }
}

fn cached_file_summary(
    ctx: &mut ScriptCompileContext,
    path: &Path,
    epoch: u64,
    imports: &mut Vec<(String, String)>,
) -> SummaryLookup {
    count(Step::SharedLock, 1);
    let Ok(cache) = FILE_TYPE_CACHE.read() else {
        return SummaryLookup::Miss;
    };
    let Some(entry) = cache.get(path) else {
        return SummaryLookup::Miss;
    };
    match entry.revalidate(path, epoch) {
        Ok(()) => {
            count(Step::Module, 1);
            ctx.merge_file_summary(&entry.summary);
            imports.extend(entry.summary.imports.iter().cloned());
            SummaryLookup::Cached(entry.summary.dependencies.clone())
        }
        Err(source) => SummaryLookup::Reload(source),
    }
}

fn merge_file_source(
    ctx: &mut ScriptCompileContext,
    path: &Path,
    source: Option<FileTypeSource>,
    epoch: u64,
    imports: &mut Vec<(String, String)>,
) -> Option<Vec<TypeDependency>> {
    let source = source?;
    let summary = file_summary_from_source(path, &source.text);
    count(Step::Module, 1);
    ctx.merge_file_summary(&summary);
    imports.extend(summary.imports.iter().cloned());
    let dependencies = summary.dependencies.clone();
    count(Step::SharedLock, 1);
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
    Some(dependencies)
}
