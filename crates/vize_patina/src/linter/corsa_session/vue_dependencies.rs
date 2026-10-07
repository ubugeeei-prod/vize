//! Real sibling SFC component types in a private Patina checker session.

use corsa::api::FileChangeSummary;
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use vize_canon::{
    ImportSourceMap, SfcTypeCheckOptions, batch::OffsetAdjustment, type_check_sfc_with_options_api,
};
use vize_l0::{FxHashMap, String};

use super::{errors::io_error_message, paths::virtual_file_path};

mod specifiers;
use specifiers::Specifiers;
#[cfg(test)]
mod tests;

pub(super) struct Prepared {
    pub(super) code: Option<String>,
    pub(super) source_map: ImportSourceMap,
    pub(super) files: Vec<(PathBuf, String)>,
}

pub(super) struct CachedDocument {
    source: String,
    generated: String,
    suffix: &'static str,
}

pub(super) fn prepare(
    source: &str,
    filename: &str,
    session_root: &Path,
    project_root: &Path,
    cache: &mut FxHashMap<PathBuf, CachedDocument>,
) -> Prepared {
    let mut prepared = Prepared {
        code: None,
        source_map: ImportSourceMap::empty(),
        files: Vec::new(),
    };
    if !source.contains(".vue") {
        cache.clear();
        return prepared;
    }
    let authored = absolute_path(Path::new(filename));
    let mut visited = FxHashMap::default();
    visited.insert(
        authored.clone(),
        virtual_file_path(session_root, project_root, filename),
    );
    let mut queue = vec![(authored, String::from(source), None)];
    while let Some((path, text, target)) = queue.pop() {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &text, SourceType::tsx()).parse();
        let mut specifiers = Specifiers::default();
        specifiers.visit_program(&parsed.program);
        let mut edits = Vec::new();
        for (start, end, specifier) in specifiers.0 {
            let relative = specifier.starts_with('.');
            let dependency = if relative {
                path.parent()
                    .unwrap_or(project_root)
                    .join(specifier.as_str())
            } else if Path::new(specifier.as_str()).is_absolute() {
                PathBuf::from(specifier.as_str())
            } else {
                continue;
            };
            let dependency = absolute_path(&dependency);
            let replacement = if specifier.ends_with(".vue") {
                if let Some(target) = visited.get(&dependency) {
                    target.clone()
                } else {
                    // A missing or invalid component remains unresolved. Do not
                    // invent a safe type merely because its suffix is `.vue`.
                    let Ok(content) = std::fs::read_to_string(&dependency) else {
                        continue;
                    };
                    if cache
                        .get(&dependency)
                        .is_none_or(|cached| cached.source.as_str() != content)
                    {
                        let Some(document) = generate(&dependency, &content) else {
                            continue;
                        };
                        cache.insert(dependency.clone(), document);
                    }
                    let Some(document) = cache.get(&dependency) else {
                        continue;
                    };
                    let mut target = dependency_target(session_root, project_root, &dependency);
                    if document.suffix == ".tsx" {
                        target.set_extension("tsx");
                    }
                    visited.insert(dependency.clone(), target.clone());
                    queue.push((dependency, document.generated.clone(), Some(target.clone())));
                    target
                }
            } else if relative {
                dependency
            } else {
                continue;
            };
            edits.push((
                start,
                end,
                String::from(replacement.to_string_lossy().replace('\\', "/")),
            ));
        }
        edits.sort_unstable_by_key(|edit| edit.0);
        let source_map = ImportSourceMap::new(
            edits
                .iter()
                .map(|(start, end, replacement)| OffsetAdjustment {
                    original_offset: *start,
                    adjustment: replacement.len() as i32 - (end - start) as i32,
                })
                .collect(),
        );
        let mut code = text;
        for (start, end, replacement) in edits.into_iter().rev() {
            code.replace_range(start as usize..end as usize, &replacement);
        }
        if let Some(target) = target {
            prepared.files.push((target, code));
        } else if source_map != ImportSourceMap::empty() {
            prepared.code = Some(code);
            prepared.source_map = source_map;
        }
    }
    cache.retain(|path, _| visited.contains_key(path));
    prepared
}

pub(super) fn materialize(
    files: &[(PathBuf, String)],
    previous: &[PathBuf],
) -> Result<FileChangeSummary, String> {
    let mut changes = FileChangeSummary {
        changed: Vec::new(),
        created: Vec::new(),
        deleted: Vec::new(),
    };
    for (path, content) in files {
        let prior = std::fs::read_to_string(path).ok();
        if prior.as_deref() == Some(content.as_str()) {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                io_error_message(
                    "Failed to create component mirror directory",
                    parent,
                    &error,
                )
            })?;
        }
        std::fs::write(path, content).map_err(|error| {
            io_error_message("Failed to write component type mirror", path, &error)
        })?;
        let document = path.to_string_lossy().as_ref().into();
        if prior.is_some() {
            changes.changed.push(document);
        } else {
            changes.created.push(document);
        }
    }
    for old in previous {
        if !files.iter().any(|(path, _)| path == old) {
            std::fs::remove_file(old).map_err(|error| {
                io_error_message("Failed to remove component type mirror", old, &error)
            })?;
            changes.deleted.push(old.to_string_lossy().as_ref().into());
        }
    }
    Ok(changes)
}

fn absolute_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    })
}

fn dependency_target(session_root: &Path, project_root: &Path, source: &Path) -> PathBuf {
    if source.strip_prefix(project_root).is_ok() {
        return virtual_file_path(session_root, project_root, &source.to_string_lossy());
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hash);
    let parent = session_root
        .join("external-components")
        .join(hash.finish().to_string());
    virtual_file_path(
        &parent,
        source.parent().unwrap_or(source),
        &source.to_string_lossy(),
    )
}

fn generate(path: &Path, source: &str) -> Option<CachedDocument> {
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).ok()?;
    let tsx = descriptor
        .script
        .iter()
        .chain(descriptor.script_setup.iter())
        .any(|block| matches!(block.lang.as_deref(), Some("tsx" | "jsx")));
    let result = type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions {
            filename: path.to_string_lossy().as_ref().into(),
            include_virtual_ts: true,
            ..Default::default()
        },
    );
    Some(CachedDocument {
        source: source.into(),
        generated: result.virtual_ts?,
        suffix: if tsx { ".tsx" } else { ".ts" },
    })
}
