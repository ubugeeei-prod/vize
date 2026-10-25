//! Standard native range hints share editor readiness and transport recovery.

use super::EditorLspSession;
use crate::lsp_client::CorsaProjectClient;
use corsa::runtime::block_on;
use serde_json::Value;
use vize_l0::{String, cstr};

struct InlayHintRequest;

impl lsp_types::request::Request for InlayHintRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/inlayHint";
}

impl EditorLspSession {
    fn inlay_hints(
        &mut self,
        document_uri: &str,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Result<Option<Value>, String> {
        trace_native_inlay("editorReadinessInput", || {
            serde_json::json!({
                "requestUri":document_uri,
                "apiAttachment":self.configured_api.as_ref().map(|api| &api.session),
                "documentGeneration":self.document_generation,
                "readyGeneration":self.ready_generation,
                "mirroredText":self.documents.get(document_uri)
            })
        });
        let uri = self.ready_document_uri(document_uri)?;
        trace_native_inlay("editorQuery", || {
            serde_json::json!({
                "requestUri":uri,"range":{"start":start,"end":end},
                "apiAttachment":self.configured_api.as_ref().map(|api| &api.session),
                "documentGeneration":self.document_generation,
                "readyGeneration":self.ready_generation,
                "mirroredText":self.documents.get(document_uri),
                "pid":std::process::id(),"nativePidExposedBySdk":false
            })
        });
        let response = block_on(self.client.request::<InlayHintRequest>(serde_json::json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": { "line": start.0, "character": start.1 },
                "end": { "line": end.0, "character": end.1 }
            }
        })))
        .map_err(|error| cstr!("Failed to request editor LSP inlay hints: {error}"));
        trace_native_inlay("editorResponse", || match &response {
            Ok(value) => serde_json::json!({"ok":true,"response":value}),
            Err(error) => serde_json::json!({"ok":false,"error":error.as_str()}),
        });
        response
    }
}

impl CorsaProjectClient {
    pub(crate) fn inlay_hints_raw(
        &mut self,
        uri: &str,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Result<Option<Value>, String> {
        trace_native_inlay("projectQuery", || {
            serde_json::json!({
                "uri":uri,"start":start,"end":end,"root":self.project_root,
                "config":self.explicit_project_config,"executable":self.executable,
                "sharedEditorTransport":self.shared_editor_transport,
                "projectSessionAttachedToEditor":self.project_session_attached_to_editor,
                "sourcePresent":self.document_texts.contains_key(uri)
            })
        });
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        let mut response =
            self.request_with_editor_lsp_recovery(|session| session.inlay_hints(uri, start, end))?;
        if let Some(value) = &mut response {
            // `data` and command arguments are opaque native payloads. Only
            // protocol label locations carry remappable document identities.
            for hint in value.as_array_mut().into_iter().flatten() {
                for part in hint
                    .get_mut("label")
                    .and_then(Value::as_array_mut)
                    .into_iter()
                    .flatten()
                {
                    if let Some(location) = part.get_mut("location") {
                        self.remap_result_uris(location);
                    }
                }
            }
        }
        Ok(response)
    }
}

pub(super) fn trace_native_inlay(stage: &str, value: impl FnOnce() -> Value) {
    if std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        tracing::info!(stage, value = %value(), "native inlay hint observation");
    }
}

#[cfg(test)]
mod trace_tests {
    use super::trace_native_inlay;
    use serde_json::{Value, json};
    use std::{cell::Cell, process::Command, sync::Mutex};
    use tracing::{Event, Metadata, Subscriber, field, span};
    use vize_l0::cstr;

    static EVENTS: Mutex<Vec<Value>> = Mutex::new(Vec::new());

    struct Capture;

    struct Fields(Value);

    impl field::Visit for Fields {
        fn record_str(&mut self, field: &field::Field, value: &str) {
            self.0[field.name()] = json!(value);
        }

        fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
            self.0[field.name()] = json!(cstr!("{value:?}").as_str());
        }
    }

    impl Subscriber for Capture {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
            span::Id::from_u64(1)
        }

        fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

        fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

        fn event(&self, event: &Event<'_>) {
            let mut fields = Fields(json!({}));
            event.record(&mut fields);
            let metadata = event.metadata();
            EVENTS.lock().unwrap().push(json!({
                "level": metadata.level().as_str(),
                "target": metadata.target(),
                "fields": fields.0
            }));
        }

        fn enter(&self, _span: &span::Id) {}

        fn exit(&self, _span: &span::Id) {}
    }

    #[test]
    fn trace_payload_is_lazy_and_informational() {
        const PROBE: &str = "VIZE_TEST_NATIVE_INLAY_TRACE_PROBE";
        if let Some(mode) = std::env::var_os(PROBE) {
            let called = Cell::new(0);
            tracing::subscriber::with_default(Capture, || {
                trace_native_inlay("startupConfigurationSnapshot", || {
                    called.set(called.get() + 1);
                    json!({"ok": true, "response": null})
                });
            });
            if mode == "enabled" {
                assert_eq!(called.get(), 1);
                assert_eq!(
                    *EVENTS.lock().unwrap(),
                    vec![json!({
                        "level": "INFO",
                        "target": "vize_canon::lsp_client::editor_lsp::inlay_hint",
                        "fields": {
                            "message": "native inlay hint observation",
                            "stage": "startupConfigurationSnapshot",
                            "value": "{\"ok\":true,\"response\":null}"
                        }
                    })]
                );
            } else {
                assert_eq!(called.get(), 0);
                assert_eq!(*EVENTS.lock().unwrap(), Vec::<Value>::new());
            }
            return;
        }

        // Child processes keep the process-global environment isolated from
        // other Rust tests. Only the literal opt-in enables payload creation.
        for (mode, setting) in [
            ("default", None),
            ("disabled", Some("0")),
            ("enabled", Some("1")),
        ] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "lsp_client::editor_lsp::inlay_hint::trace_tests::trace_payload_is_lazy_and_informational",
                    "--nocapture",
                ])
                .env(PROBE, mode)
                .env_remove("VIZE_TRACE_EDITOR_PREPARATION");
            if let Some(setting) = setting {
                command.env("VIZE_TRACE_EDITOR_PREPARATION", setting);
            }
            let output = command.output().unwrap();
            assert!(output.status.success(), "{mode}: {output:?}");
            assert!(output.stderr.is_empty(), "{mode}: {output:?}");
        }
    }

    #[test]
    fn discard_trace_payload_is_lazy_and_complete() {
        const PROBE: &str = "VIZE_TEST_EDITOR_DISCARD_TRACE_PROBE";
        if let Some(mode) = std::env::var_os(PROBE) {
            let called = Cell::new(0);
            let mut client = crate::lsp_client::CorsaProjectClient::empty_for_test(
                std::path::PathBuf::from("/trace/project"),
            );
            tracing::subscriber::with_default(Capture, || {
                for attached in [false, true] {
                    client.project_session_attached_to_editor = attached;
                    client.trace_editor_discard(|| {
                        called.set(called.get() + 1);
                        "captured caller".into()
                    });
                }
            });
            if mode == "enabled" {
                assert_eq!(called.get(), 2);
                let expected = ["independent", "attached"].map(|branch| {
                    json!({
                        "level": "INFO",
                        "target": "vize_canon::lsp_client::editor_lsp::inlay_hint",
                        "fields": {
                            "message": "native inlay hint observation",
                            "stage": "editorDiscard",
                            "value": serde_json::to_string(&json!({
                                "branch": branch,
                                "projectRoot": "/trace/project",
                                "executable": "",
                                "outerPid": std::process::id(),
                                "nativePidExposedBySdk": false,
                                "editorPresent": false,
                                "sessionPresent": false,
                                "documentCount": 0,
                                "caller": "captured caller"
                            })).unwrap()
                        }
                    })
                });
                assert_eq!(*EVENTS.lock().unwrap(), expected);
            } else {
                assert_eq!(called.get(), 0);
                assert_eq!(*EVENTS.lock().unwrap(), Vec::<Value>::new());
            }
            return;
        }
        for (mode, setting) in [
            ("default", None),
            ("disabled", Some("0")),
            ("enabled", Some("1")),
        ] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "lsp_client::editor_lsp::inlay_hint::trace_tests::discard_trace_payload_is_lazy_and_complete",
                    "--nocapture",
                ])
                .env(PROBE, mode)
                .env_remove("VIZE_TRACE_EDITOR_PREPARATION");
            if let Some(setting) = setting {
                command.env("VIZE_TRACE_EDITOR_PREPARATION", setting);
            }
            let output = command.output().unwrap();
            assert!(output.status.success(), "{mode}: {output:?}");
            assert!(output.stderr.is_empty(), "{mode}: {output:?}");
        }
    }
}
