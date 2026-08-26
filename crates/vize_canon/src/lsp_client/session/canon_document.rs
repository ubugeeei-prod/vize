//! Synchronizing an already-materialized Canon query document.

use corsa::{
    api::{FileChangeSummary, FileChanges},
    runtime::block_on,
};
use vize_s0::{String, cstr};

use crate::file_uri::file_uri_to_path;

use super::{CorsaProjectClient, uri_document_identifier};

impl CorsaProjectClient {
    pub(super) fn sync_existing_canon_document(
        &mut self,
        uri: &str,
        content: &str,
    ) -> Result<bool, String> {
        // Every Canon project owns this stub, while the shared helper file is
        // emitted only for documents whose generated code actually needs it.
        // The marker must therefore identify the materialized project, not a
        // particular host's helper shape.
        if !self.project_root.join("__vize_vue_modules.d.ts").is_file() {
            return Ok(false);
        }
        let Some(path) = file_uri_to_path(uri) else {
            return Ok(false);
        };
        if !path.starts_with(&self.project_root) || !path.is_file() {
            return Ok(false);
        }
        let previous = std::fs::read_to_string(&path).ok();
        if previous.as_deref() == Some(content) {
            return Ok(true);
        }
        std::fs::write(&path, content).map_err(|error| {
            cstr!(
                "Failed to update Canon document {}: {error}",
                path.display()
            )
        })?;
        if self.has_project_session() {
            let changes = FileChanges::Summary(FileChangeSummary {
                changed: vec![uri_document_identifier(uri)],
                created: Vec::new(),
                deleted: Vec::new(),
            });
            block_on(self.project_session_mut()?.refresh(Some(changes)))
                .map_err(|error| cstr!("Failed to refresh Canon document {uri}: {error}"))?;
        }
        Ok(true)
    }
}
