//! Redirecting relative declaration imports to their single module identity
//! when a script or SFC is materialized into canon.
//!
//! Declarations retain their authored location and module identity, including
//! declarations outside the project root. Every generated source must resolve
//! them there: silently dropping TS2307 would turn imported assertions into any.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashSet, String, cstr};

use crate::batch::virtual_project::{dependency_scan::resolve_dependency, script_virtual_path};

/// Rewrite a relative declaration specifier to the mirrored module when that
/// declaration is registered, or to its authored path when it is not.
pub(super) fn rewrite_relative_dts_specifier(
    path: &str,
    source_dir: &Path,
    project_root: &Path,
    virtual_root: &Path,
    mirrorable_project_files: Option<&FxHashSet<PathBuf>>,
) -> Option<String> {
    if !(path.starts_with("./") || path.starts_with("../")) {
        return None;
    }
    if path.ends_with(".vue") {
        return None;
    }
    // Use the same candidate order as reachability: an adjacent source file
    // must win over a declaration, and authored .js imports can resolve .d.ts.
    let resolved = resolve_dependency(path, source_dir, project_root, &[])?;
    if !is_declaration_path(&resolved) {
        return None;
    }
    let resolved = vize_carton::path::canonicalize_non_verbatim(&resolved);
    // A declaration already materialized in the virtual project must retain
    // that single identity. Importing the authored absolute path as well loads
    // ambient modules twice and produces duplicate-identifier diagnostics.
    let resolved = if mirrorable_project_files.is_some_and(|paths| paths.contains(&resolved)) {
        std::fs::read_to_string(&resolved)
            .ok()
            .and_then(|content| {
                script_virtual_path((project_root, virtual_root), &resolved, &content, false).ok()
            })
            .unwrap_or(resolved)
    } else {
        resolved
    };
    // Refer to the module rather than importing a declaration extension, which
    // TypeScript rejects with TS2846. Preserve ESM/CommonJS extension identity.
    let resolved = declaration_module_path(&resolved);
    Some(cstr!("{}", resolved.display()))
}

/// Use an extensionless .d.ts module, or the runtime spelling of .d.mts/.d.cts.
pub(crate) fn declaration_module_path(path: &Path) -> PathBuf {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.to_path_buf();
    };
    for (suffix, runtime) in [(".d.ts", ""), (".d.mts", ".mjs"), (".d.cts", ".cjs")] {
        if let Some(stem) = name.strip_suffix(suffix) {
            return path.with_file_name(cstr!("{stem}{runtime}"));
        }
    }
    path.to_path_buf()
}

fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}
