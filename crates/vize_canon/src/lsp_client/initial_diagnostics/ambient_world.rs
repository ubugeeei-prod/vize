//! Exact authority for an already mapped native compiler read.
use super::{CorsaProjectClient, DiagnosticReadView};
use crate::lsp_client::{session::uri_document_identifier, session_storage::SessionStorage};
use corsa::api::{CapabilitiesResponse, ManagedSnapshot, ProjectResponse};
use std::{path::PathBuf, sync::Arc};
use vize_l0::{FxHashMap, String};

pub(in crate::lsp_client) struct AmbientWorld {
    snapshot: Arc<ManagedSnapshot>,
    project: ProjectSelection,
    capabilities: Arc<CapabilitiesResponse>,
    project_root: PathBuf,
    explicit_config: Option<PathBuf>,
    uri: String,
    document_uri: String,
    sources: Arc<()>,
    external_document_uris: FxHashMap<String, String>,
    // An entered owner probe may outlive cancellation on the editor lane.
    _storage: Arc<SessionStorage>,
}

// A native snapshot often already owns the exact selected project, including
// all compiler options and root files. A backend may also return it separately.
enum ProjectSelection {
    Snapshot(usize),
    Separate(Box<ProjectResponse>),
}

impl DiagnosticReadView<'_> {
    pub(in crate::lsp_client) fn into_ambient_world(self, sources: Arc<()>) -> AmbientWorld {
        let project = self
            .snapshot
            .projects
            .iter()
            .position(|row| row == self.project)
            .map_or_else(
                || ProjectSelection::Separate(Box::new(self.project.clone())),
                ProjectSelection::Snapshot,
            );
        AmbientWorld {
            snapshot: self.snapshot,
            project,
            capabilities: Arc::clone(self.capabilities),
            project_root: self.project_root.clone(),
            explicit_config: self.explicit_config.clone(),
            uri: self.uri.into(),
            document_uri: self.document_uri.into(),
            sources,
            external_document_uris: self.external_document_uris.clone(),
            _storage: Arc::clone(self.storage),
        }
    }
}

impl AmbientWorld {
    fn project(&self) -> Option<&ProjectResponse> {
        match &self.project {
            ProjectSelection::Snapshot(index) => self.snapshot.projects.get(*index),
            ProjectSelection::Separate(project) => Some(project),
        }
    }

    pub(in crate::lsp_client) fn confirms_owner(&self) -> Result<bool, String> {
        crate::corsa_bridge::native_operation::checkpoint()?;
        let owner = corsa::runtime::block_on(
            self.snapshot
                .get_default_project_for_file(uri_document_identifier(&self.document_uri)),
        );
        crate::corsa_bridge::native_operation::checkpoint()?;
        Ok(matches!(owner, Ok(Some(project)) if Some(&project) == self.project()))
    }

    pub(in crate::lsp_client) fn is_current(
        &self,
        client: &CorsaProjectClient,
        uri: &str,
        document_uri: &str,
    ) -> bool {
        self.uri == uri
            && self.document_uri == document_uri
            && !client.closed
            && client.can_use_api_for_uri(uri)
            && client.session.as_ref().is_some_and(|session| {
                Arc::ptr_eq(&self.snapshot, &session.snapshot_lease())
                    && Some(session.project()) == self.project()
            })
            && Arc::ptr_eq(&self.capabilities, &client.capabilities)
            && self.project_root == client.project_root
            && self.explicit_config == client.explicit_project_config
            && client.document_texts.has_ambient_revision(&self.sources)
            && self.external_document_uris == client.external_document_uris
            && client.session_document_uris.get(uri) == Some(&self.document_uri)
    }
}
