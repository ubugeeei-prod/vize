//! Whole partial packets and existing complete ownership at an actual wait.

use super::publication_tests::{diagnostics, drain, notification};
use super::*;
use crate::ide::DiagnosticService;

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp-regressions/responsive-lint-diagnostics-8002/App.vue.txt"
);

fn original_fixture() -> Fixture {
    let fixture = fixture(false);
    fixture
        .service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(
            &json!({"lint":true,"typecheck":true,"editor":true}),
        ));
    change(&fixture, &fixture.app, 2, ORIGINAL);
    fixture
}

fn sync_packet(fixture: &Fixture) -> Value {
    let state = &fixture.service.inner().state;
    let diagnostics = DiagnosticService::collect(state, &fixture.app);
    let version = state.documents.version(&fixture.app).unwrap();
    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":fixture.app,"version":version,"diagnostics":diagnostics}})
}

#[test]
fn original_lint_and_empty_repairs_publish_while_both_collection_locks_are_held() {
    let mut fixture = original_fixture();
    let server = fixture.service.inner();
    let native = futures::executor::block_on(server.state.corsa_request_scope());
    let document = server.state.diagnostic_lock(&fixture.app);
    let collection = futures::executor::block_on(document.lock());
    let removed = ORIGINAL.replace(" alt=\"\"", "");
    for (version, source) in [(3, removed.as_str()), (4, ORIGINAL)] {
        change(&fixture, &fixture.app, version, source);
        let expected = sync_packet(&fixture);
        let observed = diagnostics(drain(&mut fixture.socket, async {
            server
                .publish_changed_sync_diagnostics(&fixture.app, version)
                .await;
        }));
        assert_eq!(observed, vec![expected]);
    }
    drop(collection);
    drop(native);
}

#[test]
fn complete_same_world_result_cannot_be_replaced_by_an_early_partial() {
    let mut fixture = original_fixture();
    let complete = current(&fixture);
    let expected = notification(&fixture, &complete);
    let server = fixture.service.inner();
    assert_eq!(
        diagnostics(drain(
            &mut fixture.socket,
            server.publish_collected_diagnostics(&fixture.app, complete)
        )),
        vec![expected]
    );
    let observed = diagnostics(drain(&mut fixture.socket, async {
        server
            .publish_changed_sync_diagnostics(&fixture.app, 2)
            .await;
    }));
    assert!(observed.is_empty());
}

#[test]
fn stale_partial_stamps_refuse_after_dependency_configuration_or_close_changes() {
    let fixture = original_fixture();
    let state = &fixture.service.inner().state;
    let document = state.diagnostic_lock(&fixture.app);
    let stale = state.corsa_request_stamp();
    change(&fixture, &fixture.unrelated, 2, UNRELATED);
    assert!(
        document
            .enqueue_sync(state, &fixture.app, 2, stale, || panic!(
                "stale dependency accepted"
            ))
            .is_none()
    );
    let stale = state.corsa_request_stamp();
    state.mark_corsa_disk_state_dirty();
    assert!(
        document
            .enqueue_sync(state, &fixture.app, 2, stale, || panic!(
                "stale environment accepted"
            ))
            .is_none()
    );
    let stale = state.corsa_request_stamp();
    state.documents.close(&fixture.app);
    assert!(
        document
            .enqueue_sync(state, &fixture.app, 2, stale, || panic!(
                "closed document accepted"
            ))
            .is_none()
    );
}

#[test]
fn cancellation_of_a_queued_partial_does_not_retire_the_complete_publication() {
    let mut fixture = original_fixture();
    let server = fixture.service.inner();
    let expected_sync = sync_packet(&fixture);
    assert!(
        server
            .client
            .publish_diagnostics(fixture.unrelated.clone(), Vec::new(), None)
            .now_or_never()
            .is_some()
    );
    let mut partial = Box::pin(server.publish_changed_sync_diagnostics(&fixture.app, 2));
    assert!(partial.as_mut().now_or_never().is_none());
    drop(partial);
    let expected_complete = notification(&fixture, &current(&fixture));
    let observed = diagnostics(drain(
        &mut fixture.socket,
        server.publish_diagnostics(&fixture.app),
    ));
    let filler = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":fixture.unrelated,"diagnostics":[]}});
    assert_eq!(observed, vec![filler, expected_sync, expected_complete]);
}
