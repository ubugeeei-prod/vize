use futures::{FutureExt, StreamExt};
use tower_lsp::{
    LanguageServer, LspService,
    lsp_types::{DidOpenTextDocumentParams, TextDocumentItem, Url},
};

use super::super::MaestroServer;

#[test]
fn did_open_defers_without_publishing_an_empty_terminal_result() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{
            "typeChecker": {
                "corsaPath": "./vize-missing-corsa-for-deferred-diagnostics"
            }
        }"#,
    )
    .unwrap();

    let (service, mut socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server.state.load_lsp_config(project.path());
    server
        .state
        .set_workspace_root(project.path().to_path_buf());

    let uri = Url::from_file_path(project.path().join("Deferred.vue")).unwrap();
    futures::executor::block_on(server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri,
            language_id: "vue".to_string(),
            version: 1,
            text: "<script setup lang=\"ts\">const answer: number = 42</script>\n".to_string(),
        },
    }));

    // The foreground path must neither initialize Corsa nor publish its empty
    // parser/lint result as though it were the terminal combined result.
    assert!(server.state.corsa_init_failure().is_none());
    assert!(socket.next().now_or_never().is_none());

    // Deferral must not silently drop validation: the background pass still
    // attempts Corsa and records the precise startup failure.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while server.state.corsa_init_failure().is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let failure = server
        .state
        .corsa_init_failure()
        .expect("background type diagnostics should attempt Corsa");
    assert!(failure.contains("spawn failed"), "{failure}");
}
