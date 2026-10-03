//! Original configured disk-file diagnostics without a second overlay snapshot.

use super::{TestResult, require};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    lsp::{LspClient, LspSpawnConfig, jsonrpc::InboundEvent},
};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use std::{
    fmt::Write,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::Duration,
};
use vize_canon::LspDiagnostic;
use vize_l0::String;

pub(super) struct DiskReference {
    // Stop the request responder before dropping the last client/process owner.
    responder: Responder,
    client: LspClient,
    session: ProjectSession,
    root: std::path::PathBuf,
}

impl DiskReference {
    pub(super) async fn spawn(
        root: &Path,
        executable: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let root = root.canonicalize()?;
        let config = root.join("tsconfig.json").canonicalize()?;
        let mode = if vize_carton::corsa_api_mode::uses_async_json_rpc_api(executable) {
            ApiMode::AsyncJsonRpcStdio
        } else {
            ApiMode::SyncMsgpackStdio
        };
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(executable.to_str().ok_or("backend path")?)
                .with_mode(mode)
                .with_cwd(&root),
            config.to_str().ok_or("reference config path")?,
            None,
        )
        .await?;
        eprintln!(
            "actual original configured ProjectSession: {}",
            serde_json::to_string(session.project())?,
        );
        let returned = Path::new(&session.project().config_file_name);
        let returned = if returned.is_absolute() {
            returned.to_path_buf()
        } else {
            root.join(returned)
        };
        require(
            returned.canonicalize()? == config,
            "actual backend resolves the configured original project",
        )?;
        let options = &session.project().compiler_options;
        require(
            options.get("moduleDetection") == Some(&serde_json::json!("force"))
                || options.get("moduleDetection") == Some(&serde_json::json!(3)),
            "actual original backend retains Module Force",
        )?;
        for option in ["strict", "allowJs", "checkJs"] {
            require(
                options.get(option).and_then(serde_json::Value::as_bool) == Some(true),
                "actual original backend retains authored checker options",
            )?;
        }
        let capabilities = session.client().describe_capabilities().await?;
        eprintln!(
            "actual original API capabilities: {}",
            serde_json::to_string(capabilities.as_ref())?,
        );
        require(
            capabilities.lsp.available,
            "actual backend supports the separate existing LSP transport",
        )?;
        let client = LspClient::spawn(LspSpawnConfig::new(executable).with_cwd(&root)).await?;
        let responder = Responder::spawn(client.clone());
        let root_uri = file_uri(&root)?;
        let initialized = client
            .request::<Initialize>(serde_json::json!({
                "processId":std::process::id(),"rootPath":root,"rootUri":root_uri,
                "workspaceFolders":[{"uri":root_uri,"name":"original-reference"}],
                "capabilities":{
                    "textDocument":{"diagnostic":{"dynamicRegistration":false,"relatedDocumentSupport":true}},
                    "workspace":{"didChangeWatchedFiles":{"dynamicRegistration":true,"relativePatternSupport":true},"diagnostic":{"refreshSupport":true}}
                },
                "initializationOptions":{"userPreferences":{"tsserver":{"automaticTypeAcquisition":{"enabled":false}}}}
            }))
            .await?;
        eprintln!("actual original LSP initialization: {initialized}");
        require(
            initialized
                .pointer("/capabilities/diagnosticProvider")
                .is_some_and(|value| !value.is_null() && value != false)
                && initialized.pointer("/capabilities/positionEncoding")
                    == Some(&serde_json::json!("utf-16")),
            "actual LSP advertises diagnostics in the original UTF16 coordinate space",
        )?;
        client.notify::<lsp_types::notification::Initialized>(lsp_types::InitializedParams {})?;
        Ok(Self {
            responder,
            client,
            session,
            root,
        })
    }

    pub(super) async fn diagnostics(
        &self,
        path: &Path,
    ) -> Result<Vec<LspDiagnostic>, Box<dyn std::error::Error>> {
        let path = path.canonicalize()?;
        require(
            path.starts_with(&self.root)
                && self.session.project().root_files.iter().any(|file| {
                    let file = Path::new(file);
                    let file = if file.is_absolute() {
                        file.to_path_buf()
                    } else {
                        self.root.join(file)
                    };
                    file.canonicalize().is_ok_and(|file| file == path)
                }),
            "original file belongs to the actual configured backend project",
        )?;
        let uri = file_uri(&path)?;
        // The existing LSP client binds the mandatory response to this exact
        // request/URI; no editor document is opened or overlaid for a disk file.
        let response = self
            .client
            .request::<PhysicalDiagnostics>(serde_json::json!({"textDocument":{"uri":uri}}))
            .await?;
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) = response
        else {
            return Err("original compiler did not return a complete diagnostic report".into());
        };
        full.full_document_diagnostic_report
            .items
            .iter()
            .map(|diagnostic| Ok(serde_json::from_value(serde_json::to_value(diagnostic)?)?))
            .collect()
    }

    pub(super) async fn close(&mut self) -> TestResult {
        let lsp = self.client.graceful_close().await;
        let api = self.session.close().await;
        let responder = self.responder.stop();
        lsp?;
        api?;
        responder?;
        Ok(())
    }
}

struct Initialize;
impl lsp_types::request::Request for Initialize {
    type Params = serde_json::Value;
    type Result = serde_json::Value;
    const METHOD: &'static str = "initialize";
}

struct PhysicalDiagnostics;
impl lsp_types::request::Request for PhysicalDiagnostics {
    type Params = serde_json::Value;
    type Result = DocumentDiagnosticReportResult;
    const METHOD: &'static str = "textDocument/diagnostic";
}

struct Responder {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Responder {
    fn spawn(client: LspClient) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let events = client.subscribe();
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                let InboundEvent::Request { id, method, params } =
                    (match events.recv_timeout(Duration::from_millis(50)) {
                        Ok(event) => event,
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
                    })
                else {
                    continue;
                };
                // Match the existing Corsa editor responder's positional
                // configuration response and registration acknowledgements.
                let response = if method.as_ref() == "workspace/configuration" {
                    let count = params
                        .get("items")
                        .and_then(serde_json::Value::as_array)
                        .map_or(0, Vec::len);
                    serde_json::Value::Array(vec![serde_json::Value::Null; count])
                } else {
                    serde_json::Value::Null
                };
                if client.respond(id, response).is_err() {
                    break;
                }
            }
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }

    fn stop(&mut self) -> TestResult {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| "original LSP responder panicked")?;
        }
        Ok(())
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn file_uri(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = path.to_str().ok_or("original URI path")?;
    let mut uri: String = "file://".into();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                uri.push(char::from(byte));
            }
            _ => write!(uri, "%{byte:02X}")?,
        }
    }
    Ok(uri)
}
