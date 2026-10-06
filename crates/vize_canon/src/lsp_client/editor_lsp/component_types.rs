//! Component classification uses Vue's actual exported type in one snapshot.

use super::{
    EditorLspSession,
    snapshot_source::{SnapshotSourceOwner, SourceTextOutcome},
};
use crate::lsp_client::{CorsaProjectClient, session::uri_document_identifier};
use corsa::{api::TypeResponse, runtime::block_on};
use serde_json::json;
use std::path::Path;
use vize_l0::{String, cstr};

mod capture;
mod shape;
mod target;

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
        let mut capture = capture::Capture::new();
        capture.record("request", || {
            json!({
                "uri":uri,"source":source,"vueModulePosition":vue_module,"positions":positions
            })
        });
        let ready = self.ready_document_uri(uri);
        capture.record("readiness", || json!({"error":ready.as_ref().err()}));
        ready?;
        if self.configured_api.is_none() {
            // Default editor sessions also need the same-process checker API.
            // Retain it before any fallible query; existing owner-first close
            // releases this attachment on shutdown, discard, and recovery.
            let attachment = self
                .attach_api()
                .map_err(super::configured_project::configuration_error);
            capture.record("attachment", || json!({"error":attachment.as_ref().err()}));
            self.configured_api = Some(attachment?);
        }
        let api = self
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Component type attachment is missing"))?;
        capture.record("retainedAttachment", || json!(api.session));
        let communication = |error| cstr!("Cannot classify component types: {error}");
        let snapshot = SnapshotSourceOwner::create(&api.client).map_err(communication);
        capture.record("snapshot", || json!({"error":snapshot.as_ref().err()}));
        let mut owner = snapshot?;
        capture.record("snapshotHandle", || json!(owner.handle()));
        let result = (|| {
            let selected = block_on(api.client.get_default_project_for_file(
                owner.handle().clone(),
                uri_document_identifier(uri),
            ))
            .map_err(communication);
            capture.record("defaultProject", || match &selected {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"error":error}),
            });
            let Some(selected) = selected? else {
                return Ok(None);
            };
            let admitted = owner
                .project(Path::new(selected.config_file_name.as_str()))
                .map_err(communication);
            capture.record("projectAdmission", || match &admitted {
                Ok(Ok(project)) => {
                    json!({"project":project.descriptor(),"sourceNames":project.source_names()})
                }
                Ok(Err(reason)) => json!({"refusal":cstr!("{reason:?}")}),
                Err(error) => json!({"error":error}),
            });
            let Ok(project) = admitted? else {
                return Ok(None);
            };
            capture.record("selectedProjectMatches", || {
                json!(project.descriptor() == &selected)
            });
            if project.descriptor() != &selected {
                return Ok(None);
            }
            let text = project.read(uri).map_err(communication);
            capture.record("sourceRead", || match &text {
                Ok(SourceTextOutcome::Complete(text)) => json!({
                    "text":text.text(),"fileName":text.file_name(),"path":text.path(),"uri":text.uri()
                }),
                Ok(SourceTextOutcome::Refused {reason,encoded}) => json!({
                    "refusal":cstr!("{reason:?}"),"encoded":encoded.as_ref().map(|value|value.as_bytes())
                }),
                Err(error) => json!({"error":error})
            });
            let SourceTextOutcome::Complete(text) = text? else {
                return Ok(None);
            };
            capture.record("sourceMatches", || json!(text.text() == source));
            if text.text() != source {
                return Ok(None);
            }
            let snapshot = owner.handle();
            let project_id = &project.descriptor().id;
            let target = target::component_target(
                &api.client,
                snapshot,
                project_id,
                uri,
                vue_module,
                &mut capture,
            )
            .map_err(communication);
            capture.record("componentTarget", || match &target {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"error":error}),
            });
            let Some(target) = target? else {
                return Ok(None);
            };
            let types = block_on(api.client.get_types_at_positions(
                snapshot.clone(),
                project_id.clone(),
                uri_document_identifier(uri),
                positions.to_vec(),
            ))
            .map_err(communication);
            capture.record("bindingTypes", || match &types {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"error":error}),
            });
            let types = types?;
            if types.len() != positions.len() {
                return Ok(None);
            }
            types
                .into_iter()
                .zip(positions)
                .map(|(value, position)| {
                    let Some(value) = value.filter(known_type) else {
                        return Ok(None);
                    };
                    let assigned = block_on(api.client.is_type_assignable_to(
                        snapshot.clone(),
                        project_id.clone(),
                        value.id.clone(),
                        target.component.id.clone(),
                    ))
                    .map_err(communication);
                    capture.record("assignability", || match &assigned {
                        Ok(value) => json!({"position":position,"result":value}),
                        Err(error) => json!({"position":position,"error":error}),
                    });
                    if !assigned? {
                        return Ok(Some(false));
                    }
                    shape::has_component_evidence(
                        &api.client,
                        snapshot,
                        project_id,
                        &value,
                        &target.option_properties,
                        &mut capture,
                    )
                    .map_err(communication)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some)
        })();
        capture.record("classification", || match &result {
            Ok(value) => json!({"result":value}),
            Err(error) => json!({"error":error}),
        });
        let cleanup = owner.release().map_err(communication);
        capture.record("release", || json!({"error":cleanup.as_ref().err()}));
        match (result, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

fn known_type(value: &TypeResponse) -> bool {
    value.flags != 0 && value.flags & ANY_OR_UNKNOWN == 0
}

impl CorsaProjectClient {
    pub(crate) fn component_types_raw(
        &mut self,
        uri: &str,
        source: &str,
        vue_module: u32,
        positions: &[u32],
    ) -> Result<Option<Vec<Option<bool>>>, String> {
        let mut capture = capture::Capture::new();
        let matched = self.document_texts.get(uri).map(String::as_str) == Some(source);
        capture.record("clientSourceAdmission", || {
            json!({
                "uri":uri,"requestedSource":source,"retainedSource":self.document_texts.get(uri),
                "matches":matched
            })
        });
        if !matched {
            return Ok(None);
        }
        let result = self.request_with_editor_lsp_recovery(|session| {
            session.component_types(uri, source, vue_module, positions)
        });
        capture.record("clientResult", || match &result {
            Ok(value) => json!({"result":value}),
            Err(error) => json!({"error":error}),
        });
        result
    }
}
