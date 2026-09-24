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
use crate::batch::materialize_fs::write_if_changed;

/// Names reserved by Canon's Vue runtime mirror. Other `@vue/*` packages may
/// come from the project's ancestor installs and must remain resolvable.
pub(in crate::batch) fn protected_vue_namespace_packages(
    project_root: &Path,
) -> Vec<vize_carton::String> {
    let Some(vue_source) = resolve_vue_package(project_root) else {
        return vec!["runtime-dom".into(), "runtime-core".into()];
    };
    match resolve_vue_runtime_packages(project_root, &vue_source) {
        VueRuntimePackages::Namespace(namespace) => std::fs::read_dir(namespace)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok().map(Into::into))
            .collect(),
        VueRuntimePackages::RuntimeDom(_) | VueRuntimePackages::Stub => vec![
            "runtime-dom".into(),
            "runtime-core".into(),
            "reactivity".into(),
        ],
    }
}

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
        // The source namespace can be a pnpm store directory. Keep the mirror
        // scope real so adding unrelated `@vue/*` packages never writes through
        // a whole-scope symlink into that store.
        ensure_stub_dir(&vue_namespace_target)?;
        for entry in std::fs::read_dir(vue_namespace_source)? {
            let entry = entry?;
            let source = entry.path();
            let target = vue_namespace_target.join(entry.file_name());
            if source.is_dir() {
                symlink_path(&source, &target)?;
            } else if source.is_file() {
                write_if_changed(&target, &std::fs::read(source)?)?;
            }
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
    let adjacent_core = resolve_adjacent_runtime_core_package(runtime_dom_source);
    let runtime_core_source = adjacent_core.as_deref().or(runtime_core_source);
    if let Some(runtime_core_source) = runtime_core_source {
        link_vue_runtime_core_package(node_modules_dir, runtime_core_source)?;
        if let Some(reactivity) = resolve_adjacent_named_package(runtime_core_source, "reactivity")
        {
            link_named_vue_package(node_modules_dir, "reactivity", &reactivity)?;
        }
        Ok(())
    } else {
        write_vue_runtime_core_stub(node_modules_dir)
    }
}

fn resolve_adjacent_named_package(package: &Path, name: &str) -> Option<PathBuf> {
    let direct = package
        .parent()
        .map(|parent| parent.join(name))
        .filter(|candidate| candidate.exists());
    if direct.is_some() {
        return direct;
    }
    std::fs::canonicalize(package)
        .ok()
        .and_then(|real| real.parent().map(|parent| parent.join(name)))
        .filter(|candidate| candidate.exists())
}

fn link_named_vue_package(
    node_modules_dir: &Path,
    name: &str,
    source: &Path,
) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    symlink_path(&package_link_source(source), &vue_namespace_dir.join(name))
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
