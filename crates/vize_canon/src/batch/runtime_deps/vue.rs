use std::path::{Path, PathBuf};

use super::resolver::{
    VueRuntimePackages, resolve_package, resolve_vue_package, resolve_vue_runtime_packages,
};
use super::stubs::{
    VUE_FACADE_JSX_GLOBAL_TYPES, VUE_FACADE_JSX_RUNTIME_TYPES, VUE_FACADE_PACKAGE_JSON,
    VUE_FACADE_TYPES, VUE_RUNTIME_CORE_STUB_PACKAGE_JSON, VUE_RUNTIME_CORE_STUB_TYPES,
    VUE_RUNTIME_DOM_STUB_PACKAGE_JSON, VUE_RUNTIME_DOM_STUB_TYPES,
};
use super::{ensure_stub_dir, package_link_source, prune_stub_dir, symlink_path};
use crate::batch::materialize_fs::{remove_path, write_if_changed};

pub(super) fn materialize_vue_support(
    project_root: &Path,
    node_modules_dir: &Path,
) -> std::io::Result<()> {
    let vue_target = node_modules_dir.join("vue");

    if let Some(vue_source) = resolve_vue_package(project_root)
        && symlink_path(&package_link_source(&vue_source), &vue_target).is_ok()
    {
        match resolve_vue_runtime_packages(project_root, &vue_source) {
            VueRuntimePackages::Namespace(vue_namespace_source) => {
                materialize_vue_namespace_packages(
                    node_modules_dir,
                    &vue_namespace_source,
                    resolve_package(project_root, "@vue/runtime-core").as_deref(),
                )?;
            }
            VueRuntimePackages::RuntimeDom(runtime_dom_source) => {
                link_vue_runtime_packages(
                    node_modules_dir,
                    &runtime_dom_source,
                    resolve_package(project_root, "@vue/runtime-core").as_deref(),
                )?;
            }
            VueRuntimePackages::Stub => write_vue_runtime_dom_stub(node_modules_dir)?,
        }
        return Ok(());
    }

    if let Some(runtime_dom_source) = resolve_package(project_root, "@vue/runtime-dom") {
        write_vue_facade(node_modules_dir)?;
        link_vue_runtime_packages(
            node_modules_dir,
            &runtime_dom_source,
            resolve_package(project_root, "@vue/runtime-core").as_deref(),
        )?;
        return Ok(());
    }

    write_vue_facade(node_modules_dir)?;
    write_vue_runtime_dom_stub(node_modules_dir)?;
    Ok(())
}

pub(crate) fn write_vue_facade(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_dir = node_modules_dir.join("vue");
    ensure_stub_dir(&vue_dir)?;
    write_if_changed(
        &vue_dir.join("package.json"),
        VUE_FACADE_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vue_dir.join("index.d.ts"), VUE_FACADE_TYPES.as_bytes())?;
    write_if_changed(
        &vue_dir.join("jsx-runtime.d.ts"),
        VUE_FACADE_JSX_RUNTIME_TYPES.as_bytes(),
    )?;
    write_if_changed(
        &vue_dir.join("jsx.d.ts"),
        VUE_FACADE_JSX_GLOBAL_TYPES.as_bytes(),
    )?;
    prune_stub_dir(
        &vue_dir,
        &["package.json", "index.d.ts", "jsx-runtime.d.ts", "jsx.d.ts"],
    )?;
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

fn materialize_vue_namespace_packages(
    node_modules_dir: &Path,
    vue_namespace_source: &Path,
    runtime_core_source: Option<&Path>,
) -> std::io::Result<()> {
    let vue_namespace_target = node_modules_dir.join("@vue");
    let runtime_dom_source = vue_namespace_source.join("runtime-dom");
    let has_runtime_dom = runtime_dom_source.exists();
    let has_runtime_core = vue_namespace_source.join("runtime-core").exists();
    if has_runtime_dom && has_runtime_core {
        if symlink_path(vue_namespace_source, &vue_namespace_target).is_err() {
            remove_path(&vue_namespace_target)?;
        }
        return Ok(());
    }
    if has_runtime_dom {
        return link_vue_runtime_packages(
            node_modules_dir,
            &runtime_dom_source,
            runtime_core_source,
        );
    }
    write_vue_runtime_dom_stub(node_modules_dir)
}

fn link_vue_runtime_packages(
    node_modules_dir: &Path,
    runtime_dom_source: &Path,
    runtime_core_source: Option<&Path>,
) -> std::io::Result<()> {
    link_vue_runtime_dom_package(node_modules_dir, runtime_dom_source)?;
    if let Some(runtime_core_source) = runtime_core_source {
        link_vue_runtime_core_package(node_modules_dir, runtime_core_source)
    } else if let Some(runtime_core_source) =
        resolve_adjacent_runtime_core_package(runtime_dom_source)
    {
        link_vue_runtime_core_package(node_modules_dir, &runtime_core_source)
    } else {
        write_vue_runtime_core_stub(node_modules_dir)
    }
}

fn resolve_adjacent_runtime_core_package(runtime_dom_source: &Path) -> Option<PathBuf> {
    runtime_dom_source
        .parent()
        .map(|parent| parent.join("runtime-core"))
        .filter(|candidate| candidate.exists())
        .or_else(|| {
            std::fs::canonicalize(runtime_dom_source)
                .ok()
                .and_then(|real_runtime_dom| {
                    real_runtime_dom
                        .parent()
                        .map(|parent| parent.join("runtime-core"))
                })
                .filter(|candidate| candidate.exists())
        })
}

fn link_vue_runtime_core_package(
    node_modules_dir: &Path,
    runtime_core_source: &Path,
) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_core_target = vue_namespace_dir.join("runtime-core");
    symlink_path(
        &package_link_source(runtime_core_source),
        &runtime_core_target,
    )
}

pub(crate) fn write_vue_runtime_dom_stub(node_modules_dir: &Path) -> std::io::Result<()> {
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
    write_vue_runtime_core_stub(node_modules_dir)?;
    Ok(())
}

fn write_vue_runtime_core_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_core_dir = vue_namespace_dir.join("runtime-core");
    ensure_stub_dir(&runtime_core_dir)?;
    write_if_changed(
        &runtime_core_dir.join("package.json"),
        VUE_RUNTIME_CORE_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(
        &runtime_core_dir.join("index.d.ts"),
        VUE_RUNTIME_CORE_STUB_TYPES.as_bytes(),
    )?;
    prune_stub_dir(&runtime_core_dir, &["package.json", "index.d.ts"])?;
    Ok(())
}
