use std::path::{Path, PathBuf};

use super::error::CorsaResult;
use super::materialize_fs::{ensure_dir, prune_dir_entries, remove_path, write_if_changed};
use vize_carton::FxHashSet;

mod resolver;
mod stubs;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use resolver::{test_env_var_os, with_test_env_overrides};

use resolver::{
    VueRuntimePackages, resolve_package, resolve_vue_package, resolve_vue_runtime_packages,
};
pub(crate) use stubs::VUE_RUNTIME_DOM_STUB_TYPES;
use stubs::{
    VITE_CLIENT_STUB, VITE_STUB_PACKAGE_JSON, VUE_FACADE_PACKAGE_JSON, VUE_FACADE_TYPES,
    VUE_RUNTIME_DOM_STUB_PACKAGE_JSON,
};

pub(super) fn materialize_runtime_dependencies(
    project_root: &Path,
    virtual_root: &Path,
) -> CorsaResult<()> {
    let node_modules_dir = virtual_root.join("node_modules");
    ensure_dir(&node_modules_dir)?;

    materialize_vue_support(project_root, &node_modules_dir)?;
    materialize_vite_support(project_root, &node_modules_dir)?;
    prune_runtime_node_modules(&node_modules_dir)?;

    Ok(())
}

fn materialize_vue_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_target = node_modules_dir.join("vue");
    let vue_namespace_target = node_modules_dir.join("@vue");

    if let Some(vue_source) = resolve_vue_package(project_root)
        && symlink_path(&package_link_source(&vue_source), &vue_target).is_ok()
    {
        match resolve_vue_runtime_packages(project_root, &vue_source) {
            VueRuntimePackages::Namespace(vue_namespace_source) => {
                if symlink_path(&vue_namespace_source, &vue_namespace_target).is_err() {
                    remove_path(&vue_namespace_target)?;
                }
            }
            VueRuntimePackages::RuntimeDom(runtime_dom_source) => {
                link_vue_runtime_dom_package(node_modules_dir, &runtime_dom_source)?;
            }
            VueRuntimePackages::Stub => write_vue_runtime_dom_stub(node_modules_dir)?,
        }
        return Ok(());
    }

    if let Some(runtime_dom_source) = resolve_package(project_root, "@vue/runtime-dom") {
        write_vue_facade(node_modules_dir)?;
        link_vue_runtime_dom_package(node_modules_dir, &runtime_dom_source)?;
        return Ok(());
    }

    write_vue_facade(node_modules_dir)?;
    write_vue_runtime_dom_stub(node_modules_dir)?;
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

fn write_vue_facade(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_dir = node_modules_dir.join("vue");
    ensure_stub_dir(&vue_dir)?;
    write_if_changed(
        &vue_dir.join("package.json"),
        VUE_FACADE_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vue_dir.join("index.d.ts"), VUE_FACADE_TYPES.as_bytes())?;
    prune_stub_dir(&vue_dir, &["package.json", "index.d.ts"])?;
    Ok(())
}

fn link_vue_runtime_dom_package(
    node_modules_dir: &Path,
    runtime_dom_source: &Path,
) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_dom_target = vue_namespace_dir.join("runtime-dom");
    symlink_path(
        &package_link_source(runtime_dom_source),
        &runtime_dom_target,
    )
}

fn write_vue_runtime_dom_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_dom_dir = vue_namespace_dir.join("runtime-dom");
    ensure_stub_dir(&runtime_dom_dir)?;
    write_if_changed(
        &runtime_dom_dir.join("package.json"),
        VUE_RUNTIME_DOM_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(
        &runtime_dom_dir.join("index.d.ts"),
        VUE_RUNTIME_DOM_STUB_TYPES.as_bytes(),
    )?;
    prune_stub_dir(&runtime_dom_dir, &["package.json", "index.d.ts"])?;
    Ok(())
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

fn prune_runtime_node_modules(node_modules_dir: &Path) -> std::io::Result<()> {
    let expected_files = FxHashSet::default();
    let preserved_roots = ["vue", "vite", "@vue"]
        .into_iter()
        .map(|name| node_modules_dir.join(name))
        .filter(|path| path.exists() || path.is_symlink())
        .collect::<Vec<_>>();
    super::materialize_fs::prune_unexpected_entries(
        node_modules_dir,
        &expected_files,
        &preserved_roots,
    )
}
