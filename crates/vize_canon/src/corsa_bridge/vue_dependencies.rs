//! Dependency graph collection for Vue editor virtual documents.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

use super::bridge::normalize_document_uri;
use super::vue_dependency_specifiers::collect_relative_ts_specifiers;
use super::vue_document::{
    CorsaVueVirtualDocumentOptions, GeneratedVueDocument, generate_vue_document,
};
use crate::batch::ImportRewriter;
use crate::file_uri::path_to_file_uri;

const VUE_DEPENDENCY_FALLBACK: &str =
    "const component: any = undefined;\nexport default component;\n";

pub(super) fn collect_dependency_documents(
    documents: &mut Vec<(String, String)>,
    host: &GeneratedVueDocument,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
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
                &parent_dir(&path),
                &content,
                source_type,
            ),
        }
    }
}

struct ImportQueue<'a> {
    documents: &'a mut Vec<(String, String)>,
    queue: &'a mut VecDeque<DependencyScan>,
    visited_vue: &'a mut FxHashSet<PathBuf>,
    visited_ts: &'a mut FxHashSet<PathBuf>,
    overlays: &'a FxHashMap<PathBuf, &'a str>,
}

enum DependencyScan {
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
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    queue_vue_imports(&mut imports, options, rewriter, dir, code, source_type);
    queue_ts_imports(&mut imports, rewriter, dir, code, source_type);
}

fn queue_vue_imports(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    for specifier in rewriter.collect_relative_vue_specifiers(code, source_type, Some(dir)) {
        let path = normalize_path(&dir.join(specifier.as_str()));
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !imports.visited_vue.insert(key) {
            continue;
        }
        let Some(content) = dependency_content(&path, imports.overlays) else {
            continue;
        };
        let generated = match generate_vue_document(&path, &content, options, rewriter) {
            Ok(generated) => generated,
            Err(_) => {
                imports.documents.push((
                    fallback_vue_virtual_uri(&path),
                    VUE_DEPENDENCY_FALLBACK.into(),
                ));
                continue;
            }
        };
        imports.documents.push((
            generated.virtual_uri.clone(),
            generated.generated.code.clone(),
        ));
        if generated.generated.virtual_suffix == ".tsx" {
            imports.documents.push(tsx_vue_import_shim(&path));
        }
        imports.queue.push_back(DependencyScan::Vue {
            dir: parent_dir(&generated.source_path),
            source_type: generated.generated.source_type,
            pre_rewrite_code: generated.generated.pre_rewrite_code,
        });
    }
}

fn fallback_vue_virtual_uri(path: &Path) -> String {
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
        let rewritten = rewriter
            .rewrite(&content, dependency_source_type, path.parent())
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

fn dependency_content(path: &Path, overlays: &FxHashMap<PathBuf, &str>) -> Option<String> {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    overlays
        .get(&key)
        .map(|content| String::from(*content))
        .or_else(|| std::fs::read_to_string(path).ok().map(Into::into))
}

fn resolve_relative_script_import(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);
    if base.extension().is_some() {
        return known_script_path(&base).then(|| normalize_path(&base));
    }

    for ext in [
        "ts", "tsx", "mts", "cts", "d.ts", "d.mts", "d.cts", "js", "jsx", "mjs", "cjs",
    ] {
        let candidate = base.with_extension(ext);
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }
    for name in [
        "index.ts",
        "index.tsx",
        "index.mts",
        "index.cts",
        "index.js",
        "index.jsx",
        "index.mjs",
        "index.cjs",
        "index.d.ts",
        "index.d.mts",
        "index.d.cts",
    ] {
        let candidate = base.join(name);
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }
    None
}

fn known_script_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.exists()
        && (name.ends_with(".ts")
            || name.ends_with(".tsx")
            || name.ends_with(".mts")
            || name.ends_with(".cts")
            || name.ends_with(".js")
            || name.ends_with(".jsx")
            || name.ends_with(".mjs")
            || name.ends_with(".cjs"))
}

fn source_type_for_path(path: &Path) -> SourceType {
    SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts())
}

fn parent_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::resolve_relative_script_import;

    #[test]
    fn resolves_directory_module_declaration_indices() {
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        std::fs::create_dir_all(src.join("esm")).expect("esm dir");
        std::fs::create_dir_all(src.join("cjs")).expect("cjs dir");

        let esm = src.join("esm").join("index.d.mts");
        let cjs = src.join("cjs").join("index.d.cts");
        let schema = src.join("schema.d.ts");
        std::fs::write(&esm, "export type Value = string;\n").expect("esm dts");
        std::fs::write(&cjs, "export type Value = string;\n").expect("cjs dts");
        std::fs::write(&schema, "export type Schema = { id: string };\n").expect("schema dts");

        assert_eq!(
            resolve_relative_script_import(&src, "./esm").as_deref(),
            Some(esm.as_path())
        );
        assert_eq!(
            resolve_relative_script_import(&src, "./cjs").as_deref(),
            Some(cjs.as_path())
        );
        assert_eq!(
            resolve_relative_script_import(&src, "./schema").as_deref(),
            Some(schema.as_path())
        );
    }
}
