//! Editor lint diagnostics see the names Nuxt auto-imports, as `vize lint` does.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "whole LSP session fixture"
)]

use futures::{FutureExt, StreamExt, channel::mpsc};
use serde_json::{Value, json};
use std::{future::Future, path::Path, time::Duration};
use tower::Service;
use tower_lsp::{LspService, lsp_types::Url};

use super::MaestroServer;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/nuxt-template-auto-imports"
);
const RULE: &str = "vue/no-undefined-refs";

/// The fixture's inputs at their runtime paths.
const PROJECT: [(&str, &str); 5] = [
    ("package.json.txt", "package.json"),
    ("nuxt.config.ts.txt", "nuxt.config.ts"),
    ("nuxt-package.json.txt", "node_modules/nuxt/package.json"),
    ("imports.d.ts.txt", ".nuxt/types/imports.d.ts"),
    ("index.vue.txt", "app/pages/index.vue"),
];

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn fixture(source: &str) -> vize_l0::String {
    std::fs::read_to_string(Path::new(FIXTURE).join(source))
        .unwrap()
        .into()
}

/// A Nuxt project built from the fixture inputs, optionally without the
/// generated declarations.
fn project(generated: bool) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(directory.path()).unwrap();
    for (source, runtime) in PROJECT {
        if generated || !runtime.starts_with(".nuxt/") {
            write(&root, runtime, &fixture(source));
        }
    }
    (directory, root)
}

struct Session {
    service: LspService<MaestroServer>,
    received: mpsc::UnboundedReceiver<Value>,
    next_id: u64,
}

impl Session {
    async fn send(&mut self, packet: Value) -> Option<Value> {
        futures::future::poll_fn(|cx| self.service.poll_ready(cx))
            .await
            .unwrap();
        self.service
            .call(serde_json::from_value(packet).unwrap())
            .await
            .unwrap()
            .map(|response| serde_json::to_value(response).unwrap())
    }

    async fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let response = self
            .send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await
            .unwrap();
        response["result"].clone()
    }

    async fn notify(&mut self, method: &str, params: Value) {
        assert_eq!(
            self.send(json!({"jsonrpc":"2.0","method":method,"params":params}))
                .await,
            None
        );
    }

    /// The next versioned diagnostics for `uri`, with the project log lines sent before it.
    async fn publication(&mut self, uri: &Url) -> (Value, Vec<Value>) {
        let mut logs = Vec::new();
        loop {
            let packet = self.received.next().await.unwrap();
            // The startup line is not about any project.
            if packet["method"] == "window/logMessage"
                && packet["params"]["message"] != "vize_maestro LSP server initialized"
            {
                logs.push(packet["params"].clone());
            }
            if packet["method"] == "textDocument/publishDiagnostics"
                && packet["params"]["uri"] == json!(uri)
                && packet["params"]["version"].is_number()
            {
                return (packet["params"]["diagnostics"].clone(), logs);
            }
        }
    }

    /// Read publications for `uri` until its findings become `expected`.
    /// Another watcher path may republish the earlier result once more first,
    /// but nothing other than `previous` may appear on the way.
    async fn settle(&mut self, uri: &Url, previous: &[Value], expected: &[Value]) -> Vec<Value> {
        let mut logs = Vec::new();
        loop {
            let (diagnostics, mut seen) = self.publication(uri).await;
            logs.append(&mut seen);
            let findings = undefined_refs(&diagnostics);
            if findings == expected {
                return logs;
            }
            assert_eq!(findings, previous);
        }
    }
}

/// Drive one server the way the stdio differential runner does.
fn session<F: Future<Output = ()>>(root: &Path, scenario: impl FnOnce(Session) -> F) {
    let (service, mut socket) = LspService::new(MaestroServer::new);
    let (messages, received) = mpsc::unbounded();
    let (stop, stopped) = futures::channel::oneshot::channel::<()>();
    let workspace = Url::from_directory_path(root).unwrap();
    crate::runtime::block_on(async move {
        let client = async move {
            let stopped = stopped.fuse();
            futures::pin_mut!(stopped);
            loop {
                futures::select! {
                    _ = stopped => break,
                    packet = socket.next().fuse() => {
                        // The transport closes once the scenario drops the server.
                        let Some(packet) = packet else { break };
                        let packet = serde_json::to_value(packet).unwrap();
                        assert_eq!(packet.get("id"), None, "no client capability was offered");
                        // A finished scenario reads nothing that arrives after it.
                        if messages.unbounded_send(packet).is_err() {
                            break;
                        }
                    }
                }
            }
        };
        let run = async move {
            let mut session = Session {
                service,
                received,
                next_id: 0,
            };
            let initialized = session
                .request(
                    "initialize",
                    json!({
                        "rootUri": workspace,
                        "capabilities": {"textDocument":{"completion":{"completionItem":{
                            "documentationFormat":["markdown","plaintext"]
                        }}}},
                        "initializationOptions": {"editor":true,"lint":true,"typecheck":false},
                        "workspaceFolders": [{"uri":workspace,"name":"project"}]
                    }),
                )
                .await;
            assert!(initialized["capabilities"].is_object());
            session.notify("initialized", json!({})).await;
            scenario(session).await;
            stop.send(()).unwrap();
        };
        let bounded = crate::runtime::timeout(Duration::from_secs(30), run);
        let (result, ()) = futures::join!(bounded, client);
        result.unwrap();
    });
}

async fn open(session: &mut Session, uri: &Url, text: &str) {
    session
        .notify(
            "textDocument/didOpen",
            json!({"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":text}}),
        )
        .await;
}

/// The rule's findings as `[line, character, message]`.
fn undefined_refs(diagnostics: &Value) -> Vec<Value> {
    diagnostics
        .as_array()
        .unwrap()
        .iter()
        .filter(|diagnostic| diagnostic["code"] == RULE)
        .map(|diagnostic| {
            json!([
                diagnostic["range"]["start"]["line"],
                diagnostic["range"]["start"]["character"],
                diagnostic["message"]
            ])
        })
        .collect()
}

fn undefined(line: u32, character: u32, name: &str) -> Value {
    json!([
        line,
        character,
        vize_l0::cstr!(
            "Variable '{name}' is not defined\n\nHelp: Define in <script setup> or ensure it's \
             imported"
        )
        .as_str()
    ])
}

#[test]
fn generated_names_are_defined_in_the_editor_and_a_typo_is_still_reported() {
    let (_directory, root) = project(true);
    let page = root.join("app/pages/index.vue");
    let uri = Url::from_file_path(&page).unwrap();
    let source = fixture("index.vue.txt");
    let expected: Value = serde_json::from_str(&fixture("responses.expected.json")).unwrap();
    session(&root, |mut session| async move {
        open(&mut session, &uri, &source).await;
        let (diagnostics, logs) = session.publication(&uri).await;
        assert_eq!(
            undefined_refs(&diagnostics),
            [undefined(5, 33, "formatCuont")]
        );
        assert_eq!(logs, Vec::<Value>::new());
        // The pinned differential responses: the typo's hover carries the
        // finding, the auto-import's does not.
        for case in expected.as_array().unwrap() {
            let hover = session
                .request(
                    "textDocument/hover",
                    json!({"textDocument":{"uri":uri},"position":case["position"]}),
                )
                .await;
            assert_eq!(hover, case["result"]);
        }
    });
}

#[test]
fn regenerated_declarations_refresh_open_documents_without_a_restart() {
    let (_directory, root) = project(false);
    // Nuxt's project references give the page its own project context, so
    // the event must refresh that context and not only the server's state.
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./.nuxt/tsconfig.app.json"}]}"#,
    );
    write(
        &root,
        ".nuxt/tsconfig.app.json",
        r#"{"include":["../app/**/*"]}"#,
    );
    let page = root.join("app/pages/index.vue");
    let uri = Url::from_file_path(&page).unwrap();
    let declarations = root.join(".nuxt/types/imports.d.ts");
    let declarations_uri = Url::from_file_path(&declarations).unwrap();
    let source = "<script setup lang=\"ts\">\nconst count = 1;\n</script>\n\n<template>\n  <p>{{ formatCount(count) }} {{ formatCuont(count) }} {{ useCart() }}</p>\n</template>\n";
    let notice = vize_l0::cstr!(
        "Skipping vue/no-undefined-refs in {}: Nuxt has not generated {} yet, so auto-imported \
         names cannot be told from undefined ones. Run `nuxi prepare` to generate it.",
        root.display(),
        declarations.display()
    );
    session(&root, |mut session| async move {
        // Before `nuxi prepare` the rule is skipped and the log says why, once.
        open(&mut session, &uri, source).await;
        let (diagnostics, logs) = session.publication(&uri).await;
        assert_eq!(undefined_refs(&diagnostics), Vec::<Value>::new());
        assert_eq!(logs, [json!({"type":3,"message":notice.as_str()})]);

        // Nuxt writes its declarations; the watcher event alone republishes.
        std::fs::create_dir_all(declarations.parent().unwrap()).unwrap();
        std::fs::write(&declarations, fixture("imports.d.ts.txt")).unwrap();
        session
            .notify(
                "workspace/didChangeWatchedFiles",
                json!({"changes":[{"uri":declarations_uri,"type":1}]}),
            )
            .await;
        let generated = [undefined(5, 33, "formatCuont"), undefined(5, 58, "useCart")];
        let logs = session.settle(&uri, &[], &generated).await;
        assert_eq!(logs, Vec::<Value>::new());

        // A new composable regenerates the file.
        let regenerated = fixture("imports.d.ts.txt").replace(
            "  const useRoute:",
            "  const useCart: typeof import('../../app/composables/cart').useCart\n  const useRoute:",
        );
        std::fs::write(&declarations, regenerated).unwrap();
        session
            .notify(
                "workspace/didChangeWatchedFiles",
                json!({"changes":[{"uri":declarations_uri,"type":2}]}),
            )
            .await;
        let logs = session
            .settle(&uri, &generated, &[undefined(5, 33, "formatCuont")])
            .await;
        assert_eq!(logs, Vec::<Value>::new());
    });
}

#[test]
fn a_project_without_nuxt_reports_every_unknown_name_as_before() {
    let (_directory, root) = project(true);
    std::fs::remove_dir_all(root.join("node_modules")).unwrap();
    let page = root.join("app/pages/index.vue");
    let uri = Url::from_file_path(&page).unwrap();
    let source = fixture("index.vue.txt");
    session(&root, |mut session| async move {
        open(&mut session, &uri, &source).await;
        let (diagnostics, logs) = session.publication(&uri).await;
        assert_eq!(
            undefined_refs(&diagnostics),
            [
                undefined(5, 8, "formatCount"),
                undefined(5, 33, "formatCuont")
            ]
        );
        assert_eq!(logs, Vec::<Value>::new());
    });
}
