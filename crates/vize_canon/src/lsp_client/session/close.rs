//! Closing an editor buffer restores its disk-backed member in the owned mirror.

use super::uri_document_identifier;
use crate::lsp_client::session_storage::{
    check_owned_destination, check_project_root, write_owned_file,
};
use crate::{
    file_uri::file_uri_to_path, lsp_client::session_paths::build_materialized_session_document_uri,
};
use corsa::api::{FileChangeSummary, FileChanges};
use std::{fs, io::ErrorKind, path::Path};
use vize_l0::{String, cstr};

pub(super) fn close_materialized_document(
    external_uri: &str,
    document_uri: &str,
    project_root: &Path,
    overlay_root: &Path,
) -> Result<Option<FileChanges>, String> {
    if document_uri == external_uri {
        return Ok(None);
    }
    if build_materialized_session_document_uri(external_uri, project_root, overlay_root).as_deref()
        != Some(document_uri)
    {
        return Err(cstr!(
            "Closed Corsa document is outside its owned mirror: {document_uri}"
        ));
    }
    check_project_root(project_root)?;
    let storage = overlay_root
        .parent()
        .ok_or_else(|| cstr!("Missing private Corsa storage owner"))?;
    let path = file_uri_to_path(document_uri)
        .ok_or_else(|| cstr!("Invalid materialized Corsa document URI {document_uri}"))?;
    let existed = check_owned_destination(storage, &path)?;
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
        write_owned_file(overlay_root, &path, &content)?;
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

#[cfg(test)]
mod tests;
