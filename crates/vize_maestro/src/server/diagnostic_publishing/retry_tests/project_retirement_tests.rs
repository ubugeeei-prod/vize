//! Even Corsa-free packets are tied to their project owner generation.
use super::{
    Url, change, current, fixture, json,
    publication_tests::{diagnostics, drain},
};
use tower_lsp::lsp_types::{FileChangeType, FileEvent};

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "the actual lint-hover cache returns Arc snapshots; identity proves no stale mutation"
)]
fn retired_unstamped_lint_and_empty_packets_cannot_publish_or_update_hover_cache() {
    for lint in [true, false] {
        let mut fixture = fixture(false);
        let config = fixture.root.path().join("vize.config.json");
        let old_config = json!({"lsp":{"typecheck":false,"lint":lint,"ecosystem":false},"linter":{"preset":"essential","rules":{"a11y/alt-text":"error"}}});
        std::fs::write(&config, old_config.to_string()).unwrap();
        let old = fixture.service.inner();
        old.state.load_workspace_config(fixture.root.path());
        change(
            &fixture,
            &fixture.app,
            2,
            "<template>\n  <img src=\"x\" />\n</template>\n",
        );
        old.state.cache_lint_hover_diagnostics(&fixture.app, 2, &[]);
        let hover_before = old.state.lint_hover_diagnostics(&fixture.app);
        assert!(hover_before.is_empty());
        let stale = current(&fixture);
        assert!(stale.stamp.is_none());
        assert_eq!(!stale.diagnostics.is_empty(), lint);
        std::fs::write(
            &config,
            json!({"lsp":{"typecheck":false,"lint":false,"ecosystem":false}}).to_string(),
        )
        .unwrap();
        let affected = old.state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(&config).unwrap(),
            typ: FileChangeType::CHANGED,
        }]);
        assert!(affected.contains(&(fixture.app.clone(), 2)));
        let fresh = futures::executor::block_on(old.for_document(&fixture.app));
        assert!(!fresh.state.project_context_retired());
        let observed = diagnostics(drain(
            &mut fixture.socket,
            fresh.publish_diagnostics_if_version(&fixture.app, 2),
        ));
        assert_eq!(
            observed,
            vec![
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":fixture.app,"version":2,"diagnostics":[]}})
            ]
        );
        let observed = diagnostics(drain(&mut fixture.socket, async {
            old.publish_collected_diagnostics(&fixture.app, stale).await;
            old.publish_initial_sync_diagnostics(&fixture.app, 2).await;
        }));
        assert!(observed.is_empty(), "{observed:#?}");
        let hover_after = old.state.lint_hover_diagnostics(&fixture.app);
        assert!(std::sync::Arc::ptr_eq(&hover_before, &hover_after));
        assert!(hover_after.is_empty());
    }
}

#[test]
fn bulk_requests_refuse_a_changed_project_generation_without_borrowing_retired_primary() {
    let fixture = fixture(false);
    let server = fixture.service.inner();
    let config = Url::from_file_path(fixture.root.path().join("vize.config.json")).unwrap();
    let result = futures::executor::block_on(server.project_request(async {
        server.state.observe_project_config_events(&[FileEvent {
            uri: config,
            typ: FileChangeType::CHANGED,
        }]);
        Ok::<_, tower_lsp::jsonrpc::Error>(42)
    }));
    assert_eq!(
        result.unwrap_err(),
        tower_lsp::jsonrpc::Error::content_modified()
    );
    assert!(server.state.project_context_retired());
    let polled = std::cell::Cell::new(false);
    let change = server.state.project_routing_change();
    let refused = futures::executor::block_on(server.project_request(async {
        polled.set(true);
        Ok::<_, tower_lsp::jsonrpc::Error>(42)
    }));
    assert_eq!(
        refused.unwrap_err(),
        tower_lsp::jsonrpc::Error::content_modified()
    );
    assert!(
        !polled.get(),
        "a request during owner mutation must not run"
    );
    drop(change);
    let polled = std::sync::atomic::AtomicBool::new(false);
    let source_mutation = server.state.documents.source_mutation_for_test();
    let refused = futures::executor::block_on(server.project_request(async {
        polled.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok::<_, tower_lsp::jsonrpc::Error>(42)
    }));
    assert_eq!(
        refused.unwrap_err(),
        tower_lsp::jsonrpc::Error::content_modified()
    );
    assert!(!polled.load(std::sync::atomic::Ordering::Relaxed));
    drop(source_mutation);
    assert_eq!(
        futures::executor::block_on(
            server.project_request(async { Ok::<_, tower_lsp::jsonrpc::Error>(42) })
        )
        .unwrap(),
        42
    );
}
