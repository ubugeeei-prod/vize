//! Preserve relative graph edges across the project's external mirror boundary.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashSet, String, cstr};

use super::{authored_vue_ts, rewrite_relative_vue_specifier, virtual_rewrite};
use crate::batch::virtual_project::{dependency_scan::resolve_dependency, mirrored_virtual_path};

pub(super) fn rewrite(
    has_external_sources: bool,
    specifier: &str,
    source_dir: Option<&Path>,
    roots: (&Path, &Path),
    mirrorable: Option<&FxHashSet<PathBuf>>,
) -> Option<String> {
    if !has_external_sources || !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let source_dir = source_dir?;
    let candidate = source_dir.join(specifier);
    // Explicit authored .vue.ts files must never resolve the generated SFC.
    let resolved = if specifier.ends_with(".vue.ts") || specifier.ends_with(".vue.tsx") {
        virtual_rewrite::resolve_source_path(&candidate)
    } else {
        resolve_dependency(specifier, source_dir, roots.0, &[])
    }?;
    let authored_resolved = resolved;
    let resolved = vize_carton::path::canonicalize_non_verbatim(&authored_resolved);
    if source_dir.starts_with(roots.0) == resolved.starts_with(roots.0)
        || mirrorable.is_some_and(|files| !files.contains(&resolved))
        || crate::batch::declaration_path::is_declaration_file(&resolved)
    {
        return None;
    }

    // Retain the authored runtime extension or directory spelling. Resolving
    // './model.js' to model.ts here would change native TypeScript semantics;
    // making an extensionless edge absolute would hide NodeNext TS2835.
    let rewritten = authored_vue_ts::rewrite_authored_or_missing_vue_import(
        specifier,
        Some(source_dir),
        true,
        false,
    )
    .or_else(|| specifier.ends_with(".vue").then(|| cstr!("{specifier}.ts")))
    .or_else(|| rewrite_relative_vue_specifier(specifier, source_dir));
    let candidate = if resolved
        .extension()
        .is_some_and(|extension| extension == "vue")
        && rewritten.is_some()
    {
        PathBuf::from(cstr!("{}.ts", resolved.display()).as_str())
    } else if matches!(
        candidate
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("js" | "jsx" | "mjs" | "cjs")
    ) {
        resolved.with_extension(candidate.extension()?)
    } else {
        let authored = canonical_candidate(&candidate, &authored_resolved, &resolved);
        let suffix = rewritten
            .as_deref()
            .and_then(|path| path.strip_prefix(specifier))
            .unwrap_or("");
        PathBuf::from(cstr!("{}{suffix}", authored.display()).as_str())
    };
    let target = mirrored_virtual_path(roots.0, roots.1, &candidate).ok()?;
    let source = mirrored_virtual_path(roots.0, roots.1, source_dir).ok()?;
    relative_specifier(&source, &target)
}

fn relative_specifier(source_dir: &Path, target: &Path) -> Option<String> {
    let common = source_dir
        .ancestors()
        .find(|ancestor| target.starts_with(ancestor))?;
    let mut relative = PathBuf::new();
    for _ in source_dir.strip_prefix(common).ok()?.components() {
        relative.push("..");
    }
    relative.push(target.strip_prefix(common).ok()?);
    let relative = relative.to_string_lossy().replace('\\', "/");
    Some(if relative.starts_with("../") {
        relative.into()
    } else {
        cstr!("./{relative}")
    })
}

fn canonical_candidate(candidate: &Path, authored_resolved: &Path, resolved: &Path) -> PathBuf {
    let implicit_extension = candidate
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| authored_resolved.file_name()?.to_str()?.strip_prefix(name))
        .filter(|suffix| suffix.starts_with('.'));
    if let Some(stem) =
        implicit_extension.and_then(|suffix| resolved.file_name()?.to_str()?.strip_suffix(suffix))
    {
        return resolved.with_file_name(stem);
    }
    vize_carton::path::physical_path(candidate)
}
