use std::path::{Path, PathBuf};

use super::error::CorsaResult;
use super::materialize_fs::{ensure_dir, prune_dir_entries, remove_path, write_if_changed};
use vize_carton::FxHashSet;

mod resolver;
mod stubs;
#[cfg(test)]
mod tests;
mod vue;

#[cfg(test)]
pub(crate) use resolver::{test_env_var_os, with_test_env_overrides};
#[cfg(test)]
pub(crate) use stubs::VUE_RUNTIME_DOM_STUB_TYPES;
#[cfg(test)]
pub(crate) use vue::{write_vue_facade, write_vue_runtime_dom_stub};

use resolver::resolve_package;
use stubs::{VITE_CLIENT_STUB, VITE_STUB_PACKAGE_JSON};
use vue::materialize_vue_support;

/// Materialize Canon's own runtime dependency entries.
///
/// `preserved_entries` are entries directly below the mirror's `node_modules`
/// that Canon owns for another reason — today, shared package shadow scopes
/// hoisted to the mirror root (#4153). Pruning them here would delete and
/// rewrite the same tree on every check and defeat warm reuse.
pub(super) fn materialize_runtime_dependencies(
    project_root: &Path,
    virtual_root: &Path,
    preserved_entries: &[PathBuf],
) -> CorsaResult<()> {
    let node_modules_dir = virtual_root.join("node_modules");
    ensure_dir(&node_modules_dir)?;

    materialize_vue_support(project_root, &node_modules_dir)?;
    materialize_vite_support(project_root, &node_modules_dir)?;
    prune_runtime_node_modules(&node_modules_dir, preserved_entries)?;

    Ok(())
}

fn materialize_vite_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_target = node_modules_dir.join("vite");

    if let Some(vite_source) = resolve_package(project_root, "vite")
        && symlink_path(&vite_source, &vite_target).is_ok()
    {
        return Ok(());
    }

    write_vite_stub(node_modules_dir)
}

fn package_link_source(source: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(source)
}

fn write_vite_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_dir = node_modules_dir.join("vite");
    ensure_stub_dir(&vite_dir)?;
    write_if_changed(
        &vite_dir.join("package.json"),
        VITE_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vite_dir.join("client.d.ts"), VITE_CLIENT_STUB.as_bytes())?;
    prune_stub_dir(&vite_dir, &["package.json", "client.d.ts"])?;
    Ok(())
}

fn ensure_stub_dir(path: &Path) -> std::io::Result<()> {
    // A link (symlink or junction) must be replaced, never written through:
    // std reports junctions as plain directories, so probe `read_link`.
    let is_link = std::fs::read_link(path).is_ok();
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !is_link => {}
        Ok(_) => {
            remove_path(path)?;
            ensure_dir(path)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ensure_dir(path)?,
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Link `target` to the canonicalized `source` package directory.
///
/// Shared with the nested-package `node_modules` mirror so both mirrors resolve
/// through the same canonicalization and idempotence rules.
pub(super) fn symlink_package_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    symlink_path(&package_link_source(source), target)
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if symlink_matches(source, target)? {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        ensure_dir(parent)?;
    }

    remove_path(target)?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }

    #[cfg(windows)]
    {
        if source.is_dir() {
            // Directory junctions need no special privilege, unlike symlinks:
            // fall back so directory links still materialize on stock Windows
            // without Developer Mode or an elevated shell.
            std::os::windows::fs::symlink_dir(source, target)
                .or_else(|_| junction::create(source, target))
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}

fn symlink_matches(source: &Path, target: &Path) -> std::io::Result<bool> {
    match std::fs::symlink_metadata(target) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    }
    // `FileType::is_symlink` is false for junctions, so probe with
    // `read_link`: it resolves both symlinks and junctions and fails for
    // real files and directories. Junction targets come back in a `\??\`
    // verbatim spelling, so compare canonicalized forms.
    let Ok(linked) = std::fs::read_link(target) else {
        return Ok(false);
    };
    Ok(linked == source || package_link_source(&linked) == package_link_source(source))
}

fn prune_stub_dir(dir: &Path, file_names: &[&str]) -> std::io::Result<()> {
    let expected_files = file_names
        .iter()
        .map(|name| dir.join(name))
        .collect::<FxHashSet<_>>();
    prune_dir_entries(dir, &expected_files)
}

fn prune_runtime_node_modules(
    node_modules_dir: &Path,
    preserved_entries: &[PathBuf],
) -> std::io::Result<()> {
    let expected_files = FxHashSet::default();
    let preserved_roots = ["vue", "vite", "@vue"]
        .into_iter()
        .map(|name| node_modules_dir.join(name))
        .chain(preserved_entries.iter().cloned())
        .filter(|path| path.exists() || path.is_symlink())
        .collect::<Vec<_>>();
    super::materialize_fs::prune_unexpected_entries(
        node_modules_dir,
        &expected_files,
        &preserved_roots,
    )
}
