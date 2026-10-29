//! Compiler-kind ABI adapter, retaining one immutable SDK lease.
//! Authority: microsoft/typescript-go@2bd066d87f5bafd315be9f40889d0a60b9e58e0b
//! internal/{api/{proto,session},ls/diagnostics,core/{compileroptions,tristate}}.go
//! (MIT, Copyright Microsoft). Only the per-file PULL union is implemented:
//! no Bind/global/options/config scope, aggregate endpoint or capability is lent.
//! The captured lease survives every RPC drain; each next RPC checks retirement.
//! The existing SINGLE worker deadline covers this read AND fallback. Text comes
//! from that same snapshot, never current disk, overlays or another snapshot.

use std::path::Path;

use corsa::{
    CorsaError,
    api::{ApiClient, ApiDialect, ManagedSnapshot, ProjectResponse},
    runtime::block_on,
};
use lsp_types::Diagnostic;
use serde_json::{Value, json};
use vize_l0::{FxHashMap, String, cstr};

use super::diagnostic_response::{NativeDiagnostic, SnapshotText, missing_method};
use crate::{
    corsa_bridge::native_operation,
    file_uri::{file_uri_to_path, path_to_typescript_name},
    lsp_client::editor_lsp::snapshot_source::{SourceTextRefusal, decode_snapshot_text},
};

enum Failure {
    Unsupported,
    Failed(String),
}

/// No mutable client access, new snapshot/project selection or state commit.
pub(in crate::lsp_client) fn read(
    client: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectResponse,
    document_uri: &str,
) -> Result<Option<Vec<Diagnostic>>, String> {
    let result = (|| {
        let path = file_uri_to_path(document_uri)
            .filter(|path| path.is_absolute())
            .ok_or(Failure::Unsupported)?;
        let file = path_to_typescript_name(&path);
        let params = json!({"snapshot":snapshot.handle,"project":project.id,"file":file});
        let derived_snapshots = client.dialect() == Some(ApiDialect::DerivedSnapshots);
        // The already-owned handshake distinguishes native 7.0's single-file
        // DTO from 7.1's files array. Never request project-wide data then filter.
        let kind_params = match client.dialect() {
            Some(ApiDialect::DerivedSnapshots) => {
                json!({"snapshot":snapshot.handle,"project":project.id,"files":[file]})
            }
            _ => params.clone(),
        };
        let mut rows = Vec::new();
        for method in required_kinds(&project.compiler_options)? {
            checkpoint()?;
            let response = block_on(client.raw_json_request(method, kind_params.clone()))
                .map_err(|error| classify(method, error))?;
            super::trace_kind(&kind_params, method, &response);
            // Native NewDiagnosticResponses(nil) serializes a successful null.
            // It is empty for THIS kind only; every remaining kind must succeed.
            if !response.is_null() {
                let diagnostics = serde_json::from_value::<Vec<NativeDiagnostic>>(response)
                    .map_err(|error| {
                        Failure::Failed(cstr!("Malformed {method} response: {error}"))
                    })?;
                if diagnostics
                    .iter()
                    .any(|row| !row.matches_dialect(derived_snapshots))
                {
                    return Err(Failure::Failed(cstr!(
                        "Malformed {method} response: beta fields on the session-snapshots dialect"
                    )));
                }
                rows.extend(diagnostics);
            }
        }
        if rows.iter().any(|row| !row.belongs_to(&file)) {
            return Err(Failure::Failed(cstr!(
                "Native file diagnostic belongs to another source"
            )));
        }
        let mut texts = FxHashMap::<String, SnapshotText>::default();
        for name in rows.iter().flat_map(NativeDiagnostic::source_names) {
            if texts.contains_key(name) {
                continue;
            }
            if !Path::new(name).is_absolute() || name.contains('\0') {
                return Err(Failure::Failed(cstr!("Invalid native source identity")));
            }
            checkpoint()?;
            let encoded =
                block_on(client.get_source_file(snapshot.handle.clone(), project.id.clone(), name))
                    .map_err(|error| classify("getSourceFile", error))?
                    .ok_or(Failure::Unsupported)?;
            let text = match decode_snapshot_text(&encoded, name) {
                Ok(text) => text,
                Err(SourceTextRefusal::Version) => return Err(Failure::Unsupported),
                Err(reason) => {
                    return Err(Failure::Failed(cstr!(
                        "Malformed native snapshot source {name}: {reason:?}"
                    )));
                }
            };
            super::trace_source(&params, name, text);
            texts.insert(
                name.into(),
                SnapshotText::new(text).map_err(Failure::Failed)?,
            );
        }
        checkpoint()?;
        rows.into_iter()
            .map(|row| {
                row.into_lsp(&file, |name| {
                    texts
                        .get(name)
                        .ok_or_else(|| cstr!("Missing native snapshot source {name}"))
                })
                .map_err(Failure::Failed)
            })
            .collect::<Result<Vec<_>, _>>()
    })();
    match result {
        Ok(rows) => Ok(Some(rows)),
        Err(Failure::Unsupported) => Ok(None),
        Err(Failure::Failed(error)) => Err(error),
    }
}

fn checkpoint() -> Result<(), Failure> {
    native_operation::checkpoint().map_err(Failure::Failed)
}

fn required_kinds(options: &Value) -> Result<Vec<&'static str>, Failure> {
    let options = options.as_object().ok_or_else(|| {
        Failure::Failed(cstr!("Native project compiler options are not an object"))
    })?;
    let mut declarations = false;
    for key in ["declaration", "composite"] {
        match options.get(key) {
            None | Some(Value::Null) | Some(Value::Bool(false)) => {}
            Some(Value::Bool(true)) => declarations = true,
            Some(_) => {
                return Err(Failure::Failed(cstr!(
                    "Invalid native {key} compiler option"
                )));
            }
        }
    }
    let mut methods = vec![
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
    ];
    if declarations {
        methods.push("getDeclarationDiagnostics");
    }
    Ok(methods)
}

fn classify(method: &str, error: CorsaError) -> Failure {
    if missing_method(method, &error) {
        Failure::Unsupported
    } else {
        Failure::Failed(cstr!("Failed native {method} read: {error}"))
    }
}

#[cfg(test)]
mod tests;
