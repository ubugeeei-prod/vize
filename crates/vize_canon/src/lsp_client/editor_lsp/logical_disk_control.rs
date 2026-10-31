//! Native authored-disk oracle. No CorsaProjectClient overlay path is used.

use super::{EditorLspSession, project_identity::ProjectIdentityError};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::{String, cstr};

pub(in crate::lsp_client) fn run(
    executable: &Path,
    root: &Path,
    uri: &str,
    main: &str,
    capture: &Path,
    phase: &str,
) -> Result<Value, String> {
    let mut packet = json!({"phase":phase,"root":root,"uri":uri,"main":main,
        "scope":"fresh standard LSP; physical dependency reads without a support overlay",
        "supportDisk":fs::read_to_string(root.join("src/support.ts")).unwrap(),
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
        let opened = editor.mirror(uri, main).map(|_| ());
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
        let hover = editor.hover(uri, 2, 32);
        packet["hoverResult"] = serde_json::to_value(&hover).unwrap();
        save(&packet, "hover")?;
        let hover = hover?;
        let definition = editor.definition(uri, 2, 32);
        packet["definitionResult"] = serde_json::to_value(&definition).unwrap();
        save(&packet, "definition")?;
        let definition = definition?;
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
