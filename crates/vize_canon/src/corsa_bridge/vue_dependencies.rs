//! Dependency graph collection for Vue editor virtual documents.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

use super::bridge::normalize_document_uri;
use super::vue_dependency_paths::{normalize_path, resolve_relative_script_import};
use super::vue_dependency_specifiers::collect_relative_ts_specifiers;
use super::vue_document::{CorsaVueVirtualDocumentOptions, GeneratedVueDocument};
use crate::batch::ImportRewriter;
use crate::file_uri::path_to_file_uri;

const VUE_DEPENDENCY_FALLBACK: &str =
    "const component: any = undefined;\nexport default component;\n";

pub(super) fn collect_dependency_documents(
    documents: &mut Vec<(String, String)>,
    host: &GeneratedVueDocument,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    alias_context: &super::vue_dependencies_alias::AliasContext,
    overlays: &FxHashMap<PathBuf, &str>,
) {
    let mut visited_vue = FxHashSet::<PathBuf>::default();
    visited_vue.insert(host.source_path.clone());
    let mut visited_ts = FxHashSet::<PathBuf>::default();
    let mut queue = VecDeque::<DependencyScan>::new();
    queue.push_back(DependencyScan::Vue {
        dir: parent_dir(&host.source_path),
        source_type: host.generated.source_type,
        pre_rewrite_code: host.generated.pre_rewrite_code.clone(),
    });

    while let Some(scan) = queue.pop_front() {
        match scan {
            DependencyScan::Vue {
                dir,
                source_type,
                pre_rewrite_code,
            } => queue_imports(
                ImportQueue {
                    documents,
                    queue: &mut queue,
                    visited_vue: &mut visited_vue,
                    visited_ts: &mut visited_ts,
                    overlays,
                },
                options,
                rewriter,
                alias_context,
                &dir,
                &pre_rewrite_code,
                source_type,
            ),
            DependencyScan::Script {
                path,
                source_type,
                content,
            } => queue_imports(
                ImportQueue {
                    documents,
                    queue: &mut queue,
                    visited_vue: &mut visited_vue,
                    visited_ts: &mut visited_ts,
                    overlays,
                },
                options,
                rewriter,
                alias_context,
                &parent_dir(&path),
                &content,
                source_type,
            ),
        }
    }
}

pub(super) struct ImportQueue<'a> {
    pub(super) documents: &'a mut Vec<(String, String)>,
    pub(super) queue: &'a mut VecDeque<DependencyScan>,
    pub(super) visited_vue: &'a mut FxHashSet<PathBuf>,
    pub(super) visited_ts: &'a mut FxHashSet<PathBuf>,
    pub(super) overlays: &'a FxHashMap<PathBuf, &'a str>,
}

pub(super) enum DependencyScan {
    Vue {
        dir: PathBuf,
        source_type: SourceType,
        pre_rewrite_code: String,
    },
    Script {
        path: PathBuf,
        source_type: SourceType,
        content: String,
    },
}

fn queue_imports(
    mut imports: ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    alias_context: &super::vue_dependencies_alias::AliasContext,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    queue_vue_imports(
        &mut imports,
        options,
        rewriter,
        alias_context,
        dir,
        code,
        source_type,
    );
    queue_ts_imports(
        &mut imports,
        rewriter,
        alias_context,
        dir,
        code,
        source_type,
    );
    super::vue_dependencies_alias::queue_alias_imports(
        &mut imports,
        options,
        rewriter,
        alias_context,
        dir,
        code,
        source_type,
    );
}

fn queue_vue_imports(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    alias_context: &super::vue_dependencies_alias::AliasContext,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    for specifier in rewriter.collect_relative_vue_specifiers(code, source_type, Some(dir)) {
        let path = normalize_path(&dir.join(specifier.as_str()));
        queue_vue_dependency(imports, options, rewriter, alias_context, &path);
    }
}

/// Generate one resolved `.vue` dependency, push its document, and queue the
/// component's own imports. Shared with the alias walk so both spellings of a
/// dependency produce the same document and the same ambient fallback.
pub(super) fn queue_vue_dependency(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    alias_context: &super::vue_dependencies_alias::AliasContext,
    path: &Path,
) {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !imports.visited_vue.insert(key) {
        return;
    }
    let Some(content) = dependency_content(path, imports.overlays) else {
        return;
    };
    let generated = match super::vue_document::generate_vue_document_with_alias(
        path,
        &content,
        options,
        rewriter,
        alias_context,
    ) {
        Ok(generated) => generated,
        Err(_) => {
            imports.documents.push((
                fallback_vue_virtual_uri(path),
                VUE_DEPENDENCY_FALLBACK.into(),
            ));
            return;
        }
    };
    imports.documents.push((
        generated.virtual_uri.clone(),
        generated.generated.code.clone(),
    ));
    if generated.generated.virtual_suffix == ".tsx" {
        imports.documents.push(tsx_vue_import_shim(path));
    }
    imports.queue.push_back(DependencyScan::Vue {
        dir: parent_dir(&generated.source_path),
        source_type: generated.generated.source_type,
        pre_rewrite_code: generated.generated.pre_rewrite_code,
    });
}

pub(super) fn fallback_vue_virtual_uri(path: &Path) -> String {
    let virtual_path = path.with_file_name(cstr!(
        "{}.ts",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    ));
    path_to_file_uri(&virtual_path)
}

pub(super) fn tsx_vue_import_shim(path: &Path) -> (String, String) {
    let shim_path = path.with_file_name(cstr!(
        "{}.ts",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    ));
    let target_name = shim_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| cstr!("{name}x"))
        .unwrap_or_else(|| "component.vue.tsx".into());
    (
        path_to_file_uri(&shim_path),
        cstr!(
            "export {{ default }} from \"./{target_name}\";\nexport * from \"./{target_name}\";\n"
        ),
    )
}

fn queue_ts_imports(
    imports: &mut ImportQueue<'_>,
    rewriter: &ImportRewriter,
    alias_context: &super::vue_dependencies_alias::AliasContext,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    for specifier in collect_relative_ts_specifiers(code, source_type) {
        let Some(path) = resolve_relative_script_import(dir, specifier.as_str()) else {
            continue;
        };
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !imports.visited_ts.insert(key) {
            continue;
        }
        let Some(content) = dependency_content(&path, imports.overlays) else {
            continue;
        };
        let dependency_source_type = source_type_for_path(&path);
        let script_dir = parent_dir(&path);
        let rewritten = rewriter
            .rewrite_with_alias_resolver(&content, dependency_source_type, path.parent(), &|spec| {
                alias_context.resolve_specifier_to_mirror_path(spec, &script_dir)
            })
            .code;
        let uri = normalize_document_uri(path_to_file_uri(&path).as_str());
        imports.documents.push((uri, rewritten));
        imports.queue.push_back(DependencyScan::Script {
            path: path.clone(),
            source_type: dependency_source_type,
            content,
        });
    }
}

pub(super) fn dependency_content(
    path: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
) -> Option<String> {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    overlays
        .get(&key)
        .map(|content| String::from(*content))
        .or_else(|| std::fs::read_to_string(path).ok().map(Into::into))
}

pub(super) fn source_type_for_path(path: &Path) -> SourceType {
    SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts())
}

pub(super) fn parent_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}
