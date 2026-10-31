//! Refresh the existing API owner after acknowledging the complete revision.

use crate::lsp_client::CorsaProjectClient;
use corsa::api::ApiDialect;
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn refresh_logical_attached_project(&mut self) -> Result<(), String> {
        if !self.project_session_attached_to_editor
            || !self.session.as_ref().is_some_and(|session| {
                session.client().dialect() == Some(ApiDialect::DerivedSnapshots)
            })
        {
            return Ok(());
        }
        let uri = self
            .document_texts
            .keys()
            .min()
            .cloned()
            .ok_or_else(|| cstr!("Logical API refresh requires a retained document barrier"))?;
        // Reuse the existing full editor synchronization and response-backed
        // barrier before refreshing this same snapshot lineage. Refresh drops
        // no connection and preserves any independently retained old lease.
        self.refresh_acknowledged_editor_snapshot(&uri)
    }
}
