//! Diagnostics-oriented `ApiClient` methods.

use super::{
    ApiClient, DocumentIdentifier, FileDiagnosticsResponse, ProjectDiagnosticsResponse,
    SnapshotDiagnosticsResponse,
    requests_core::{SnapshotFileRequest, SnapshotProjectRequest},
};
use crate::{CorsaError, Result};

impl ApiClient {
    /// Returns diagnostics for every project in a snapshot.
    pub async fn get_diagnostics_for_snapshot(
        &self,
        snapshot: super::SnapshotHandle,
    ) -> Result<SnapshotDiagnosticsResponse> {
        let value = self
            .raw_json_request(
                "getDiagnosticsForSnapshot",
                serde_json::json!({ "snapshot": snapshot }),
            )
            .await
            .map_err(|error| {
                ApiClient::map_missing_method(
                    error,
                    "snapshot diagnostics are not supported by this runtime; check describeCapabilities before requesting diagnostics",
                )
            })?;
        if value.is_null() {
            return Err(CorsaError::Unsupported(
                "snapshot diagnostics returned no data; check describeCapabilities before requesting diagnostics",
            ));
        }
        Ok(serde_json::from_value(value)?)
    }

    /// Returns diagnostics for every file in a project.
    pub async fn get_diagnostics_for_project(
        &self,
        snapshot: super::SnapshotHandle,
        project: super::ProjectHandle,
    ) -> Result<ProjectDiagnosticsResponse> {
        let value = self
            .raw_json_request(
                "getDiagnosticsForProject",
                serde_json::to_value(SnapshotProjectRequest { snapshot, project })?,
            )
            .await
            .map_err(|error| {
                ApiClient::map_missing_method(
                    error,
                    "project diagnostics are not supported by this runtime; check describeCapabilities before requesting diagnostics",
                )
            })?;
        if value.is_null() {
            return Err(CorsaError::Unsupported(
                "project diagnostics returned no data; check describeCapabilities before requesting diagnostics",
            ));
        }
        Ok(serde_json::from_value(value)?)
    }

    /// Returns diagnostics for a single file in a project.
    pub async fn get_diagnostics_for_file(
        &self,
        snapshot: super::SnapshotHandle,
        project: super::ProjectHandle,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<FileDiagnosticsResponse> {
        let mut params_value = serde_json::to_value(SnapshotFileRequest {
            snapshot,
            file: file.into(),
        })?;
        let Some(params) = params_value.as_object_mut() else {
            return Err(CorsaError::Protocol(
                "file diagnostics request must serialize to an object".into(),
            ));
        };
        params.insert("project".into(), serde_json::to_value(project)?);
        let value = self
            .raw_json_request("getDiagnosticsForFile", params_value)
            .await
            .map_err(|error| {
                ApiClient::map_missing_method(
                    error,
                    "file diagnostics are not supported by this runtime; check describeCapabilities before requesting diagnostics",
                )
            })?;
        if value.is_null() {
            return Err(CorsaError::Unsupported(
                "file diagnostics returned no data; check describeCapabilities before requesting diagnostics",
            ));
        }
        Ok(serde_json::from_value(value)?)
    }
}
