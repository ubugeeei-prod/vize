use super::virtual_overlay;
use crate::file_uri::{file_uri_to_path, path_to_file_uri};
use std::path::{Component, Path, PathBuf};
use vize_s0::String;

pub(super) fn build_session_document_uri(
    uri: &str,
    project_root: &Path,
    overlay_confirmed: bool,
) -> String {
    let Some(external_path) = external_document_path(uri) else {
        return uri.into();
    };

    // Keep real in-project files at real paths. Virtual mirrors (`*.vue.ts`,
    // `*.html.ts`) can also stay there when Corsa overlay support is confirmed;
    // otherwise materialize them under the session overlay root so imports
    // never point at non-existent workspace files.
    if external_path.starts_with(project_root)
        && (external_path.exists()
            || (overlay_confirmed && virtual_overlay::target_exists(&external_path)))
    {
        return path_to_file_uri(&external_path);
    }

    path_to_file_uri(&materialized_session_path(&external_path, project_root))
}

pub(super) fn build_materialized_session_document_uri(
    uri: &str,
    project_root: &Path,
) -> Option<String> {
    let external_path = external_document_path(uri)?;
    Some(path_to_file_uri(&materialized_session_path(
        &external_path,
        project_root,
    )))
}

fn materialized_session_path(external_path: &Path, project_root: &Path) -> PathBuf {
    let mut session_path = overlay_root_for_project(project_root);
    let canonical_external = external_path.parent().and_then(|parent| {
        let file_name = external_path.file_name()?;
        parent
            .canonicalize()
            .ok()
            .map(|parent| parent.join(file_name))
    });
    let relative = external_path.strip_prefix(project_root).ok().or_else(|| {
        canonical_external
            .as_deref()
            .and_then(|path| path.strip_prefix(project_root).ok())
    });
    if let Some(relative) = relative {
        session_path.push(relative);
        return session_path;
    }

    for component in external_path.components() {
        match component {
            Component::Prefix(prefix) => session_path.push(prefix.as_os_str()),
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => session_path.push("__parent__"),
            Component::Normal(part) => session_path.push(part),
        }
    }

    session_path
}

pub(super) fn overlay_root_for_project(project_root: &Path) -> PathBuf {
    if is_under_node_modules_vize(project_root) {
        return project_root.join("overlays");
    }

    project_root
        .join("node_modules")
        .join(".vize")
        .join("corsa-overlay")
}

fn is_under_node_modules_vize(path: &Path) -> bool {
    let mut previous = None;
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            continue;
        };
        if previous == Some("node_modules") && name == ".vize" {
            return true;
        }
        previous = Some(name);
    }
    false
}

pub(super) fn external_document_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = file_uri_to_path(uri) {
        return Some(path);
    }

    let (scheme, path) = uri.split_once("://")?;
    let mut session_path = PathBuf::from("__scheme__");
    session_path.push(scheme);
    session_path.push(path.trim_start_matches('/'));
    Some(session_path)
}
