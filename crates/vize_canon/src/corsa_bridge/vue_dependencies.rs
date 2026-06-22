//! Dependency graph collection for Vue editor virtual documents.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString};

use super::bridge::normalize_document_uri;
use super::vue_document::{
    CorsaVueVirtualDocumentOptions, GeneratedVueDocument, generate_vue_document,
};
use crate::batch::ImportRewriter;
use crate::file_uri::path_to_file_uri;

pub(super) fn collect_dependency_documents(
    documents: &mut Vec<(String, String)>,
    host: &GeneratedVueDocument,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
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
    queue_ts_imports(&mut imports, dir, code, source_type);
}

fn queue_vue_imports(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    for specifier in rewriter.collect_relative_vue_specifiers(code, source_type) {
        let path = normalize_path(&dir.join(specifier.as_str()));
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !imports.visited_vue.insert(key) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(generated) = generate_vue_document(&path, &content, options, rewriter) else {
            continue;
        };
        imports.documents.push((
            generated.virtual_uri.clone(),
            generated.generated.code.clone(),
        ));
        imports.queue.push_back(DependencyScan::Vue {
            dir: parent_dir(&generated.source_path),
            source_type: generated.generated.source_type,
            pre_rewrite_code: generated.generated.pre_rewrite_code,
        });
    }
}

fn queue_ts_imports(
    imports: &mut ImportQueue<'_>,
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
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let uri = normalize_document_uri(path_to_file_uri(&path).as_str());
        imports.documents.push((uri, content.to_compact_string()));
        imports.queue.push_back(DependencyScan::Script {
            path: path.clone(),
            source_type: source_type_for_path(&path),
            content: content.into(),
        });
    }
}

fn collect_relative_ts_specifiers(code: &str, source_type: SourceType) -> Vec<String> {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Expression, Statement};
    use oxc_ast_visit::Visit;
    use oxc_parser::Parser;

    let allocator = Allocator::default();
    let result = Parser::new(&allocator, code, source_type).parse();
    let mut specifiers = Vec::new();
    let mut push = |path: &str| {
        if (path.starts_with("./") || path.starts_with("../"))
            && !path.ends_with(".vue")
            && !path.ends_with(".vue.ts")
            && !specifiers.iter().any(|known| known == path)
        {
            specifiers.push(path.to_compact_string());
        }
    };

    for stmt in &result.program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => push(&decl.source.value),
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    push(&source.value);
                }
            }
            Statement::ExportAllDeclaration(decl) => push(&decl.source.value),
            _ => {}
        }
    }

    struct DynamicImportCollector {
        imports: Vec<String>,
    }
    impl<'a> Visit<'a> for DynamicImportCollector {
        fn visit_import_expression(&mut self, expr: &oxc_ast::ast::ImportExpression<'a>) {
            if let Expression::StringLiteral(lit) = &expr.source {
                self.imports.push(lit.value.as_str().to_compact_string());
            }
            oxc_ast_visit::walk::walk_import_expression(self, expr);
        }
    }

    let mut collector = DynamicImportCollector {
        imports: Vec::new(),
    };
    collector.visit_program(&result.program);
    for path in collector.imports {
        push(&path);
    }

    specifiers
}

fn resolve_relative_script_import(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);
    if base.extension().is_some() {
        return known_script_path(&base).then(|| normalize_path(&base));
    }

    for ext in ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"] {
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
