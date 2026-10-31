use super::virtual_overlay;
use crate::file_uri::{file_uri_to_path, path_to_file_uri};
use std::path::{Component, Path, PathBuf};
use vize_l0::String;

pub(super) fn build_session_document_uri(
    uri: &str,
    project_root: &Path,
    overlay_root: &Path,
    overlay_confirmed: bool,
) -> String {
    let Some(external_path) = external_document_path(uri) else {
        return uri.into();
    };

    // Keep real in-project files at real paths. Virtual mirrors (`*.vue.ts`,
    // `*.html.ts`) can also stay there when Corsa overlay support is confirmed;
    // otherwise materialize them under the session overlay root so imports
    // never point at non-existent workspace files.
    if is_within_project_namespace(&external_path, project_root)
        && (external_path.exists()
            || (overlay_confirmed && virtual_overlay::target_exists(&external_path)))
    {
        return path_to_file_uri(&external_path);
    }

    path_to_file_uri(&materialized_session_path(
        &external_path,
        project_root,
        overlay_root,
    ))
}

/// Compare the same logical file namespace used by protocol URIs. Windows
/// canonical roots may use a verbatim prefix while their file URIs do not.
/// This does not resolve another file or change the authored cwd/config path.
pub(super) fn is_within_project_namespace(path: &Path, root: &Path) -> bool {
    if path.to_str().is_none() || root.to_str().is_none() {
        return false;
    }
    let path = file_uri_to_path(&path_to_file_uri(path));
    let root = file_uri_to_path(&path_to_file_uri(root));
    matches!((path, root), (Some(path), Some(root)) if path.starts_with(root))
}

pub(super) fn build_materialized_session_document_uri(
    uri: &str,
    project_root: &Path,
    overlay_root: &Path,
) -> Option<String> {
    let external_path = external_document_path(uri)?;
    Some(path_to_file_uri(&materialized_session_path(
        &external_path,
        project_root,
        overlay_root,
    )))
}

fn materialized_session_path(
    external_path: &Path,
    project_root: &Path,
    overlay_root: &Path,
) -> PathBuf {
    let mut session_path = overlay_root.to_path_buf();
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
    if let Some(relative) = relative.filter(|path| {
        path.components()
            .all(|part| matches!(part, Component::Normal(_)))
    }) {
        session_path.push(relative);
        return session_path;
    }

    for component in external_path.components() {
        match component {
            Component::Prefix(prefix) => {
                use sha2::Digest as _;
                let digest: String = sha2::Sha256::digest(prefix.as_os_str().as_encoded_bytes())
                    .iter()
                    .map(|byte| vize_l0::cstr!("{byte:02x}"))
                    .collect();
                session_path.push(vize_l0::cstr!("__prefix_{digest}").as_str());
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => session_path.push("__parent__"),
            Component::Normal(part) => session_path.push(part),
        }
    }

    session_path
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
