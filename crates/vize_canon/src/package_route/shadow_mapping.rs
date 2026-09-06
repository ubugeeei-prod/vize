//! Deterministic authored-source mapping for native package-shadow locations.

use std::path::{Component, Path, PathBuf};

use super::{PackageRoute, PackageRouteSource};
use vize_carton::cstr;

pub(super) fn source_for_native_shadow_path<'a>(
    route: &'a PackageRoute,
    shadow_root: &Path,
    shadow_path: &Path,
) -> Option<&'a PathBuf> {
    let selected = shadow_path.strip_prefix(shadow_root).ok()?;

    // Package source shadows are inserted before manifest-target companions.
    // Therefore a real `index.ts` is the authority for `index.ts`, even when a
    // runtime `index.js` also has an authored `index.vue` fallback. This is the
    // same `entry.or_insert` contract used by package_shadow materialization.
    if let Some(source) = direct_source_shadow(route, selected) {
        return Some(source);
    }
    if let Some(source) = route.source_targets.iter().find_map(|source| {
        (source
            .native_probe_relative_path(&route.package_root)
            .as_deref()
            == Some(selected)
            || source
                .declaration_target_relative_path(&route.package_root)
                .as_deref()
                == Some(selected))
        .then_some(&source.source_path)
    }) {
        return Some(source);
    }
    if let Some(source) = workspace_dependency_shadow(route, selected) {
        return Some(source);
    }
    for nested in &route.nested_routes {
        let Some(package_name) = nested.package_name.as_deref() else {
            continue;
        };
        if let Some(source) = source_for_native_shadow_path(
            nested,
            &shadow_root.join("node_modules").join(package_name),
            shadow_path,
        ) {
            return Some(source);
        }
    }
    None
}

fn workspace_dependency_shadow<'a>(
    route: &'a PackageRoute,
    selected: &Path,
) -> Option<&'a PathBuf> {
    route.source_targets.iter().find_map(|source| {
        route.dependency_paths.iter().find(|dependency| {
            source
                .workspace_dependency_probe_relative_path(&route.package_root, dependency)
                .as_deref()
                == Some(selected)
        })
    })
}

fn direct_source_shadow<'a>(route: &'a PackageRoute, selected: &Path) -> Option<&'a PathBuf> {
    let sources = route
        .source_paths
        .iter()
        .chain(route.dependency_paths.iter())
        .collect::<Vec<_>>();
    if let Some(source) = sources.iter().copied().find(|source_path| {
        let Ok(relative) = source_path.strip_prefix(&route.package_root) else {
            return false;
        };
        source_path
            .extension()
            .is_none_or(|extension| extension != "vue")
            && relative == selected
    }) {
        return Some(source);
    }
    sources.into_iter().find(|source_path| {
        let Ok(relative) = source_path.strip_prefix(&route.package_root) else {
            return false;
        };
        vue_shadow_relatives(relative)
            .iter()
            .any(|relative| relative == selected)
    })
}

fn vue_shadow_relatives(relative: &Path) -> [PathBuf; 3] {
    let name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let stem = relative
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    [
        parent.join(cstr!("{name}.ts").as_str()),
        parent.join(cstr!("{name}.tsx").as_str()),
        parent.join(cstr!("{stem}.d.vue.ts").as_str()),
    ]
}

impl PackageRouteSource {
    pub(crate) fn native_probe_relative_path(&self, package_root: &Path) -> Option<PathBuf> {
        self.native_probe_path
            .strip_prefix(package_root)
            .ok()
            .map(Path::to_path_buf)
    }

    pub(crate) fn declaration_target_relative_path(&self, package_root: &Path) -> Option<PathBuf> {
        let relative = self.target_path.strip_prefix(package_root).ok()?;
        is_declaration_path(relative).then(|| relative.to_path_buf())
    }

    pub(crate) fn workspace_dependency_probe_relative_path(
        &self,
        package_root: &Path,
        dependency_path: &Path,
    ) -> Option<PathBuf> {
        if dependency_path
            .extension()
            .is_some_and(|extension| extension == "vue")
        {
            return None;
        }
        let source_relative = self.source_path.strip_prefix(package_root).ok()?;
        let probe_relative = self.native_probe_path.strip_prefix(package_root).ok()?;
        if source_relative == probe_relative {
            return None;
        }
        let dependency_relative = dependency_path.strip_prefix(package_root).ok()?;
        let source_root = first_normal_component(source_relative)?;
        if source_root != "src" {
            return None;
        }
        let output_root = first_normal_component(probe_relative)?;
        if !matches!(output_root, "built" | "dist" | "lib") {
            return None;
        }
        let dependency_suffix = dependency_relative
            .strip_prefix(Path::new(source_root))
            .ok()?;
        if dependency_suffix.as_os_str().is_empty() {
            return None;
        }
        Some(Path::new(output_root).join(dependency_suffix))
    }
}

fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

fn first_normal_component(path: &Path) -> Option<&str> {
    match path.components().next()? {
        Component::Normal(part) => part.to_str(),
        _ => None,
    }
}
