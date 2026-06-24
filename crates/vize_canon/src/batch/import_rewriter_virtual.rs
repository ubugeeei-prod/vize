use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, cstr};

use super::DynamicImportCollector;

const SOURCE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".vue", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
];

pub(super) fn absolute_import_needs_virtual_rewrite(path: &Path) -> bool {
    let Some(source_path) = resolve_source_path(path) else {
        return false;
    };
    source_needs_virtual_rewrite(&source_path)
}

fn source_needs_virtual_rewrite(path: &Path) -> bool {
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue = vec![path.to_path_buf()];

    while let Some(file) = queue.pop() {
        if !visited.insert(file.clone()) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            return true;
        }

        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Some(dir) = file.parent() else {
            continue;
        };

        for specifier in collect_import_specifiers(&source) {
            let candidate = Path::new(specifier.as_str());
            let resolved = if is_relative_specifier(&specifier) {
                resolve_source_path(&dir.join(specifier.as_str()))
            } else if candidate.is_absolute() {
                resolve_source_path(candidate)
            } else {
                None
            };
            let Some(resolved) = resolved else {
                continue;
            };
            if is_node_modules_path(&resolved) {
                continue;
            }
            if resolved
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("vue")
            {
                return true;
            }
            if !visited.contains(&resolved) {
                queue.push(resolved);
            }
        }
    }

    false
}

fn resolve_source_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }

    for ext in SOURCE_EXTENSIONS {
        let candidate = append_extension(path, ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for ext in SOURCE_EXTENSIONS {
        let candidate = path.join(cstr!("index{ext}").as_str());
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

pub(super) fn append_extension(path: &Path, extension: &str) -> PathBuf {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => path.with_file_name(cstr!("{name}{extension}")),
        None => path.to_path_buf(),
    }
}

pub(super) fn is_rewritable_project_specifier(path: &Path) -> bool {
    if path
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == "node_modules")
    {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| {
            matches!(
                extension,
                "vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
            )
        })
}

pub(super) fn is_rewritable_vue_specifier(path: &str) -> bool {
    path.ends_with(".vue")
        && (path.starts_with("./")
            || path.starts_with("../")
            || path.starts_with("@/")
            || path.starts_with("~/")
            || Path::new(path).is_absolute())
}

fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

fn is_node_modules_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == std::ffi::OsStr::new("node_modules"))
}

fn collect_import_specifiers(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source, SourceType::tsx());
    let result = parser.parse();
    let mut specifiers: Vec<String> = Vec::new();

    for stmt in &result.program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                specifiers.push(decl.source.value.as_str().into())
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    specifiers.push(source.value.as_str().into());
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                specifiers.push(decl.source.value.as_str().into());
            }
            _ => {}
        }
    }

    let mut collector = DynamicImportCollector::new();
    collector.visit_program(&result.program);
    for (_, _, path) in collector.imports {
        specifiers.push(path);
    }

    specifiers
}
