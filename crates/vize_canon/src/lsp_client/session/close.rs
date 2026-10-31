//! Closing an editor buffer restores its disk-backed member in the owned mirror.

use super::uri_document_identifier;
use crate::{
    file_uri::file_uri_to_path, lsp_client::session_paths::build_materialized_session_document_uri,
};
use corsa::api::{FileChangeSummary, FileChanges};
use std::{fs, io::ErrorKind, io::Write, path::Component, path::Path};
use vize_l0::{String, cstr};

pub(super) fn close_materialized_document(
    external_uri: &str,
    document_uri: &str,
    project_root: &Path,
) -> Result<Option<FileChanges>, String> {
    if document_uri == external_uri {
        return Ok(None);
    }
    if build_materialized_session_document_uri(external_uri, project_root).as_deref()
        != Some(document_uri)
    {
        return Err(cstr!(
            "Closed Corsa document is outside its owned mirror: {document_uri}"
        ));
    }
    let path = file_uri_to_path(document_uri)
        .ok_or_else(|| cstr!("Invalid materialized Corsa document URI {document_uri}"))?;
    let existed = check_owned_destination(project_root, &path)?;
    let disk = file_uri_to_path(external_uri)
        .map(|source| {
            if !fs::metadata(&source)?.is_file() {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "closed Corsa source is not a regular file",
                ));
            }
            fs::read(source)
        })
        .transpose()
        .or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error)
            }
        })
        .map_err(|error| cstr!("Failed to read closed Corsa document {external_uri}: {error}"))?;
    let mut summary = FileChangeSummary::default();
    if let Some(content) = disk {
        let parent = path
            .parent()
            .ok_or_else(|| cstr!("Materialized Corsa document has no parent"))?;
        fs::create_dir_all(parent)
            .map_err(|error| cstr!("Failed to prepare closed Corsa document: {error}"))?;
        check_owned_destination(project_root, &path)?;
        // Replacing the leaf also preserves any original sharing its inode.
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| cstr!("Failed to prepare closed Corsa document: {error}"))?;
        temporary
            .write_all(&content)
            .and_then(|()| {
                temporary
                    .persist(&path)
                    .map(|_| ())
                    .map_err(|error| error.error)
            })
            .map_err(|error| cstr!("Failed to restore closed Corsa document: {error}"))?;
        if existed {
            summary.changed.push(uri_document_identifier(document_uri));
        } else {
            summary.created.push(uri_document_identifier(document_uri));
        }
    } else if existed {
        fs::remove_file(&path)
            .map_err(|error| cstr!("Failed to remove closed Corsa document: {error}"))?;
        summary.deleted.push(uri_document_identifier(document_uri));
    } else {
        return Ok(None);
    }
    Ok(Some(FileChanges::Summary(summary)))
}

fn check_owned_destination(project_root: &Path, path: &Path) -> Result<bool, String> {
    // Constructors retain a canonical root. Refuse a later replacement of
    // that boundary before inspecting any child through a foreign tree.
    let root = fs::symlink_metadata(project_root)
        .map_err(|error| cstr!("Failed to inspect closed Corsa project root: {error}"))?;
    if !root.is_dir()
        || root.file_type().is_symlink()
        || (cfg!(windows) && fs::read_link(project_root).is_ok())
    {
        return Err(cstr!(
            "Closed Corsa project root is not an owned real directory"
        ));
    }
    let relative = path
        .strip_prefix(project_root)
        .map_err(|_| cstr!("Closed Corsa document is outside its project"))?;
    let mut current = project_root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(cstr!("Invalid closed Corsa document path"));
        }
        current.push(component);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(cstr!("Failed to inspect closed Corsa document: {error}")),
        };
        if metadata.file_type().is_symlink() || (cfg!(windows) && fs::read_link(&current).is_ok()) {
            return Err(cstr!("Closed Corsa document crosses a linked mirror path"));
        }
        if current == path {
            if !metadata.is_file() {
                return Err(cstr!("Closed Corsa document mirror is not a file"));
            }
            return Ok(true);
        }
        if !metadata.is_dir() {
            return Err(cstr!(
                "Closed Corsa document mirror parent is not a directory"
            ));
        }
    }
    Err(cstr!(
        "Closed Corsa document cannot replace its project root"
    ))
}

#[cfg(test)]
mod tests;
