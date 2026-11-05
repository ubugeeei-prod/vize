//! Native authored-disk oracle. No CorsaProjectClient overlay path is used.

use super::{EditorLspSession, project_identity::ProjectIdentityError};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::{String, cstr};

pub(crate) fn run(
    executable: &Path,
    root: &Path,
    uri: &str,
    capture: &Path,
    phase: &str,
    query: (u32, u32),
    dependency: &Path,
) -> Result<Value, String> {
    let main_path = crate::file_uri::file_uri_to_path(uri)
        .ok_or_else(|| cstr!("Physical native control requires a file URI: {uri}"))?;
    let main = fs::read_to_string(&main_path)
        .map_err(|error| cstr!("Cannot read physical native main: {error}"))?;
    let dependency_uri = crate::file_uri::path_to_file_uri(dependency);
    let mut packet = json!({"phase":phase,"root":root,"uri":uri,"main":main,
        "scope":"fresh standard LSP; physical dependency reads without a support overlay",
        "dependencyPhysicalPath":dependency,"dependencyCallerUri":dependency_uri,
        "supportDisk":fs::read_to_string(dependency).unwrap(),
        "authoredConfig":serde_json::from_slice::<Value>(&fs::read(root.join("tsconfig.json")).unwrap()).unwrap()});
    let save = |packet: &Value, stage: &str| -> Result<(), String> {
        let path = capture.join(cstr!("logical-disk-control-{phase}-{stage}.json").as_str());
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| cstr!("Cannot retain disk control: {error}"))?;
        serde_json::to_writer_pretty(file, packet)
            .map_err(|error| cstr!("Cannot encode disk control: {error}"))
    };
    let mut editor = EditorLspSession::spawn(executable.to_str().unwrap(), root, root)?;
    let result = (|| {
        // Only the real main file is opened, with bytes identical to disk.
        // Support is intentionally never mirrored; each fresh native owner
        // must resolve its current physical bytes under the authored config.
        let opened = editor.mirror(uri, &main).map(|_| ());
        packet["openResult"] = serde_json::to_value(&opened).unwrap();
        save(&packet, "open")?;
        opened?;
        let diagnostics = editor.diagnostics(uri);
        packet["diagnosticsResult"] = serde_json::to_value(&diagnostics).unwrap();
        save(&packet, "diagnostics")?;
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) =
            diagnostics?
        else {
            return Err(cstr!("Whole native disk diagnostic report required"));
        };
        let hover = editor.hover(uri, query.0, query.1);
        packet["hoverResult"] = serde_json::to_value(&hover).unwrap();
        save(&packet, "hover")?;
        let hover = hover?;
        let definition = editor.definition(uri, query.0, query.1);
        packet["definitionResult"] = serde_json::to_value(&definition).unwrap();
        save(&packet, "definition")?;
        let definition = definition?;
        let projected = authored_definition(definition.as_ref(), dependency, &dependency_uri);
        packet["definitionIdentityProjection"] = serde_json::to_value(&projected).unwrap();
        save(&packet, "definition-identity")?;
        let (definition, _) = projected?;
        let config = editor
            .diagnosing_configuration(uri)
            .map_err(|error| match error {
                ProjectIdentityError::Communication(error) => error,
                ProjectIdentityError::Unconfigured => {
                    cstr!("Native disk control has no configured project")
                }
            });
        packet["diagnosingConfiguration"] = serde_json::to_value(&config).unwrap();
        save(&packet, "configuration")?;
        Ok(
            json!({"phase":phase,"diagnostics":report.full_document_diagnostic_report.items,
            "hover":hover,"definition":definition,"diagnosingConfiguration":config}),
        )
    })();
    let shutdown = editor.shutdown();
    packet["shutdownResult"] = serde_json::to_value(&shutdown).unwrap();
    save(&packet, "shutdown")?;
    match (result, shutdown) {
        (Ok(packet), Ok(())) => Ok(packet),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

fn authored_definition(
    definition: Option<&Value>,
    dependency: &Path,
    caller_uri: &str,
) -> Result<(Value, Value), String> {
    let canonical = dependency
        .canonicalize()
        .map_err(|error| cstr!("Cannot identify known disk target: {error}"))?;
    let physical = |uri: &str| {
        let path = crate::file_uri::file_uri_to_path(uri)
            .ok_or_else(|| cstr!("Known native disk target requires a file URI: {uri}"))?;
        path.canonicalize()
            .map_err(|error| cstr!("Cannot identify disk target URI: {error}"))
    };
    if physical(caller_uri)? != canonical {
        return Err(cstr!("Caller URI does not own the known disk target"));
    }
    let Some(Value::Array(definitions)) = definition else {
        return Err(cstr!("Whole native disk definition array is required"));
    };
    let [Value::Object(location)] = definitions.as_slice() else {
        return Err(cstr!("Exactly one native disk definition is required"));
    };
    let native_uri = location
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("Native disk definition URI is missing"))?;
    if physical(native_uri)? != canonical {
        return Err(cstr!(
            "Native definition does not name the known disk target"
        ));
    }
    let mut projected = location.clone();
    projected.insert("uri".into(), json!(caller_uri));
    Ok((
        json!([projected]),
        json!({"nativeUri":native_uri,"callerUri":caller_uri,
            "canonicalTarget":canonical,"fields":["uri"]}),
    ))
}

#[cfg(test)]
mod tests {
    use super::authored_definition;
    use serde_json::{Value, json};
    use std::fs;

    #[test]
    fn native_disk_definition_uses_only_known_caller_identity() {
        let input: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/native-document-identity-11586/disk-definition-uri.json"
        )))
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let target = root.join("support.ts");
        let foreign = root.join("foreign.ts");
        fs::create_dir(root.join("other")).unwrap();
        let shadow = root.join("other/support.ts");
        for path in [&target, &foreign, &shadow] {
            fs::write(path, "export const value = 1").unwrap();
        }
        let caller = crate::file_uri::path_to_file_uri(&target);
        let native = format!("{}support%2Ets", caller.strip_suffix("support.ts").unwrap());
        let raw = json!([{"uri":native,"range":input["range"],
            "data":{"uri":native},"detail":native}]);
        let saved = raw.clone();
        let (projected, receipt) = authored_definition(Some(&raw), &target, &caller).unwrap();
        let mut expected = raw.clone();
        expected[0]["uri"] = json!(caller);
        assert_eq!(projected, expected);
        assert_eq!(raw, saved, "the original native packet remains exact");
        assert_ne!(projected, raw, "raw spelling is not the caller contract");
        assert_eq!(receipt["fields"], json!(["uri"]));
        for case in input["rejects"].as_array().unwrap() {
            let mut candidate = raw.clone();
            let mut caller = caller.clone();
            let candidate = match case.as_str().unwrap() {
                "missing-definition" => None,
                "null-definition" => Some(Value::Null),
                "missing-entry" => Some(json!([])),
                "extra-entry" => Some(json!([raw[0], raw[0]])),
                "missing-uri" => {
                    candidate[0].as_object_mut().unwrap().remove("uri");
                    Some(candidate)
                }
                "foreign-target" | "same-name-other-file" => {
                    let file = if case == "foreign-target" {
                        &foreign
                    } else {
                        &shadow
                    };
                    candidate[0]["uri"] = json!(crate::file_uri::path_to_file_uri(file));
                    Some(candidate)
                }
                "wrong-caller-target" => {
                    caller = crate::file_uri::path_to_file_uri(&foreign);
                    Some(candidate)
                }
                unknown => panic!("unknown rejection: {unknown}"),
            };
            assert!(
                authored_definition(candidate.as_ref(), &target, &caller).is_err(),
                "{case}"
            );
        }
    }
}
