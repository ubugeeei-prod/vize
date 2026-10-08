//! Actual publishing survives unrelated revisions and cancelled collection.
#![expect(
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "whole process and LSP notification fixture"
)]

use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use futures::{FutureExt, StreamExt};
use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{
    ClientSocket, LspService,
    jsonrpc::Request,
    lsp_types::{TextDocumentContentChangeEvent, Url},
};

use super::{CollectedDiagnostics, MaestroServer};

mod publication_tests;
mod sync_feedback_tests;

const APP: &str =
    include_str!("../../../../../tests/_fixtures/lsp-corsa-responsiveness-8012/App.vue.txt");
const TOAST: &str =
    include_str!("../../../../../tests/_fixtures/lsp-corsa-responsiveness-8012/useToast.ts");
const UNRELATED: &str = "<script setup lang=\"ts\">const unrelated = 1;</script>\n";

struct Fixture {
    root: tempfile::TempDir,
    service: LspService<MaestroServer>,
    socket: ClientSocket,
    app: Url,
    unrelated: Url,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop_backend();
    }
}

impl Fixture {
    fn stop_backend(&self) {
        if let Ok(pids) = std::fs::read_to_string(self.root.path().join("backend.pids")) {
            let _ = std::process::Command::new("kill")
                .args(pids.lines())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}

fn fixture(held: bool) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("App.vue"), APP).unwrap();
    std::fs::write(root.path().join("useToast.ts"), TOAST).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"moduleResolution":"Bundler"},"include":["*.ts","*.vue"]}"#).unwrap();
    let backend = root
        .path()
        .join(if held { "held-corsa" } else { "missing-corsa" });
    if held {
        std::fs::write(
            &backend,
            "#!/bin/sh\necho $$ >> backend.pids\nexec cat 3>&1 > /dev/null\n",
        )
        .unwrap();
        std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    std::fs::write(root.path().join("vize.config.json"), json!({"lsp":{"typecheck":true,"lint":false,"ecosystem":false},"typeChecker":{"corsaPath":backend,"lspRequestTimeoutMs":60_000}}).to_string()).unwrap();
    let (mut service, socket) = LspService::new(MaestroServer::new);
    let request: Request = serde_json::from_value(json!({"jsonrpc":"2.0","id":8012,"method":"initialize","params":{"rootUri":Url::from_file_path(root.path()).unwrap(),"capabilities":{}}})).unwrap();
    let response = futures::executor::block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service.call(request).await.unwrap().unwrap()
    });
    let response = serde_json::to_value(response).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 8012);
    assert!(response.get("error").is_none());
    let app = Url::from_file_path(root.path().join("App.vue")).unwrap();
    let unrelated = Url::from_file_path(root.path().join("Unrelated.vue")).unwrap();
    let state = &service.inner().state;
    assert!(state.is_lsp_typecheck_enabled());
    for (uri, source) in [(&app, APP), (&unrelated, UNRELATED)] {
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(uri, source);
    }
    assert!(super::super::importers::open_typecheck_dependents(state, &unrelated).is_empty());
    Fixture {
        root,
        service,
        socket,
        app,
        unrelated,
    }
}

fn change(fixture: &Fixture, uri: &Url, version: i32, text: &str) {
    let state = &fixture.service.inner().state;
    assert!(state.documents.apply_changes(
        uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_owned()
        }],
        version
    ));
    state.update_virtual_docs(uri, text);
}

fn current(fixture: &Fixture) -> CollectedDiagnostics {
    let server = fixture.service.inner();
    let lock = server.state.diagnostic_lock(&fixture.app);
    futures::executor::block_on(async {
        let _guard = lock.lock().await;
        server
            .collect_diagnostics_unlocked(&fixture.app, None)
            .await
            .unwrap()
    })
}

fn whole(fixture: &Fixture, collected: &CollectedDiagnostics) -> Value {
    json!({"uri":fixture.app,"version":collected.version,"diagnostics":collected.diagnostics})
}

fn await_condition(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "original diagnostic fixture did not settle"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn receive_whole(fixture: &mut Fixture) -> Value {
    let mut result = None;
    await_condition(|| {
        if let Some(Some(request)) = fixture.socket.next().now_or_never()
            && request.method() == "textDocument/publishDiagnostics"
            && request.params().is_some_and(|params| {
                params["uri"] == fixture.app.as_str() && params.get("version").is_some()
            })
        {
            result = request.params().cloned();
        }
        result.is_some()
    });
    result.unwrap()
}

#[test]
fn cancelled_real_collection_retains_current_diagnostics_after_an_unrelated_edit() {
    let mut fixture = fixture(false);
    let server = fixture.service.inner();
    let scope = futures::executor::block_on(server.state.corsa_request_scope());
    let mut pending = Box::pin(server.publish_diagnostics(&fixture.app));
    assert!(pending.as_mut().now_or_never().is_none());
    change(
        &fixture,
        &fixture.unrelated,
        2,
        "<template>changed unrelated source</template>\n",
    );
    drop(pending);
    drop(scope);
    let expected = whole(&fixture, &current(&fixture));
    assert_eq!(receive_whole(&mut fixture), expected);
}

#[test]
fn stale_publication_requeues_the_whole_current_version_instead_of_losing_it() {
    let mut fixture = fixture(false);
    let collected = current(&fixture);
    let updated = APP.replace("App</button>", "Current App</button>");
    change(&fixture, &fixture.app, 2, &updated);
    futures::executor::block_on(
        fixture
            .service
            .inner()
            .publish_collected_diagnostics(&fixture.app, collected),
    );
    assert!(fixture.socket.next().now_or_never().is_none());
    let expected = whole(&fixture, &current(&fixture));
    assert_eq!(expected["version"], 2);
    assert_eq!(receive_whole(&mut fixture), expected);
}

#[test]
fn active_worker_without_a_sender_retains_a_rejected_unrelated_source_batch() {
    let mut fixture = fixture(true);
    assert!(
        fixture
            .service
            .inner()
            .schedule_initial_diagnostics(fixture.app.clone(), 1)
    );
    await_condition(|| fixture.root.path().join("backend.pids").exists());
    change(
        &fixture,
        &fixture.unrelated,
        2,
        "<template>unrelated editor change</template>\n",
    );
    fixture.stop_backend();
    await_condition(|| fixture.service.inner().state.corsa_init_failure().is_some());
    let expected = whole(&fixture, &current(&fixture));
    assert_eq!(receive_whole(&mut fixture), expected);
}

#[test]
fn rejected_batches_do_not_reschedule_closed_or_disabled_documents() {
    for closed in [true, false] {
        let mut fixture = fixture(false);
        let collected = current(&fixture);
        if closed {
            fixture.service.inner().state.documents.close(&fixture.app);
        } else {
            fixture
                .service
                .inner()
                .state
                .apply_lsp_initialization_options(Some(&json!({"typecheck":false})));
        }
        futures::executor::block_on(
            fixture
                .service
                .inner()
                .publish_collected_diagnostics(&fixture.app, collected),
        );
        assert!(
            !fixture
                .service
                .inner()
                .retry_current_diagnostics(&fixture.app)
        );
        assert!(fixture.socket.next().now_or_never().is_none());
    }
}
