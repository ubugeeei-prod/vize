//! Component classification uses Vue's actual exported type in one snapshot.

use super::{
    EditorLspSession,
    snapshot_source::{SnapshotSourceOwner, SourceTextOutcome},
};
use crate::lsp_client::{CorsaProjectClient, session::uri_document_identifier};
use corsa::{
    api::{ApiClient, ProjectHandle, SnapshotHandle, TypeResponse},
    runtime::block_on,
};
use std::path::Path;
use vize_l0::{String, cstr};

// Native 7.0.2 checker/ast flags. These test unavailable type evidence, not names.
const ANY_OR_UNKNOWN: u32 = (1 << 0) | (1 << 1);
const MODULE: u32 = (1 << 9) | (1 << 10);
const ALIAS: u32 = 1 << 21;

impl EditorLspSession {
    fn component_types(
        &mut self,
        uri: &str,
        source: &str,
        vue_module: u32,
        positions: &[u32],
    ) -> Result<Option<Vec<Option<bool>>>, String> {
        self.ready_document_uri(uri)?;
        if self.configured_api.is_none() {
            // Default editor sessions also need the same-process checker API.
            // Retain it before any fallible query; existing owner-first close
            // releases this attachment on shutdown, discard, and recovery.
            self.configured_api = Some(
                self.attach_api()
                    .map_err(super::configured_project::configuration_error)?,
            );
        }
        let api = self
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Component type attachment is missing"))?;
        let communication = |error| cstr!("Cannot classify component types: {error}");
        let mut owner = SnapshotSourceOwner::create(&api.client).map_err(communication)?;
        let result = (|| {
            let Some(selected) = block_on(api.client.get_default_project_for_file(
                owner.handle().clone(),
                uri_document_identifier(uri),
            ))
            .map_err(communication)?
            else {
                return Ok(None);
            };
            let Ok(project) = owner
                .project(Path::new(selected.config_file_name.as_str()))
                .map_err(communication)?
            else {
                return Ok(None);
            };
            if project.descriptor() != &selected {
                return Ok(None);
            }
            let SourceTextOutcome::Complete(text) = project.read(uri).map_err(communication)?
            else {
                return Ok(None);
            };
            if text.text() != source {
                return Ok(None);
            }
            let snapshot = owner.handle();
            let project_id = &project.descriptor().id;
            let Some(target) = component_target(&api.client, snapshot, project_id, uri, vue_module)
                .map_err(communication)?
            else {
                return Ok(None);
            };
            let types = block_on(api.client.get_types_at_positions(
                snapshot.clone(),
                project_id.clone(),
                uri_document_identifier(uri),
                positions.to_vec(),
            ))
            .map_err(communication)?;
            if types.len() != positions.len() {
                return Ok(None);
            }
            types
                .into_iter()
                .map(|value| {
                    let Some(value) = value.filter(known_type) else {
                        return Ok(None);
                    };
                    block_on(api.client.is_type_assignable_to(
                        snapshot.clone(),
                        project_id.clone(),
                        value.id,
                        target.id.clone(),
                    ))
                    .map(Some)
                    .map_err(communication)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some)
        })();
        let cleanup = owner.release().map_err(communication);
        match (result, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

fn known_type(value: &TypeResponse) -> bool {
    value.flags != 0 && value.flags & ANY_OR_UNKNOWN == 0
}

fn component_target(
    api: &ApiClient,
    snapshot: &SnapshotHandle,
    project: &ProjectHandle,
    uri: &str,
    position: u32,
) -> corsa::Result<Option<TypeResponse>> {
    let Some(module) = block_on(api.get_symbol_at_position(
        snapshot.clone(),
        project.clone(),
        uri_document_identifier(uri),
        position,
    ))?
    else {
        return Ok(None);
    };
    if module.flags & MODULE == 0 {
        return Ok(None);
    }
    let mut exports = block_on(api.get_exports_of_symbol_in_project(
        snapshot.clone(),
        project.clone(),
        module.id,
    ))?
    .into_iter()
    .filter(|symbol| symbol.name == "Component");
    let Some(mut symbol) = exports.next() else {
        return Ok(None);
    };
    if exports.next().is_some() {
        return Ok(None);
    }
    if symbol.flags & ALIAS != 0 {
        let Some(resolved) =
            block_on(api.get_aliased_symbol(snapshot.clone(), project.clone(), symbol.id))?
        else {
            return Ok(None);
        };
        symbol = resolved;
    }
    Ok(
        block_on(api.get_declared_type_of_symbol(snapshot.clone(), project.clone(), symbol.id))?
            .filter(known_type),
    )
}

impl CorsaProjectClient {
    pub(crate) fn component_types_raw(
        &mut self,
        uri: &str,
        source: &str,
        vue_module: u32,
        positions: &[u32],
    ) -> Result<Option<Vec<Option<bool>>>, String> {
        if self.document_texts.get(uri).map(String::as_str) != Some(source) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.component_types(uri, source, vue_module, positions)
        })
    }
}
