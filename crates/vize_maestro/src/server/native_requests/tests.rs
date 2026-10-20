//! Complete stale-result and cancellation laws at the real dispatch boundary.
use futures::{FutureExt, channel::oneshot, executor::block_on};
use serde_json::{Value, json};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use tower_lsp::{LanguageServer, jsonrpc::Error};

use super::super::build_lsp_service;

const APP: &str =
    include_str!("../../../../../tests/_fixtures/lsp-corsa-responsiveness-8012/App.vue.txt");
const TOAST: &str =
    include_str!("../../../../../tests/_fixtures/lsp-corsa-responsiveness-8012/useToast.ts");

#[test]
fn local_responses_keep_the_complete_reply_without_waiting_for_the_native_scope() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let uri = Url::parse("file:///project/src/App.vue").unwrap();
    server
        .state
        .documents
        .open(uri, APP.into(), 1, "vue".into());
    let active = block_on(server.state.corsa_request_scope());
    assert_eq!(
        server.local_request(|| Ok(pending_native_reply())),
        Ok(pending_native_reply())
    );
    drop(active);
}

#[test]
fn local_responses_refuse_source_close_configuration_and_retirement_changes() {
    #[derive(Debug)]
    enum Mutation {
        Source,
        Close,
        Configuration,
        Retirement,
    }
    for mutation in [
        Mutation::Source,
        Mutation::Close,
        Mutation::Configuration,
        Mutation::Retirement,
    ] {
        let (service, _socket) = build_lsp_service();
        let server = service.inner();
        let uri = Url::parse("file:///project/src/App.vue").unwrap();
        server
            .state
            .documents
            .open(uri.clone(), APP.into(), 1, "vue".into());
        let response = server.local_request(|| {
            match mutation {
                Mutation::Source => {
                    server
                        .state
                        .documents
                        .open(uri.clone(), TOAST.into(), 2, "vue".into());
                }
                Mutation::Close => {
                    server.state.documents.close(&uri);
                }
                Mutation::Configuration => server
                    .state
                    .apply_lsp_initialization_options(Some(&json!({"hover":false}))),
                Mutation::Retirement => server.state.retire_project_contexts(),
            }
            Ok(pending_native_reply())
        });
        assert_eq!(response, Err(Error::content_modified()), "{mutation:?}");
    }
}

fn pending_native_reply() -> Value {
    // Include an edit outside the request root: stale responses must be
    // refused whole, rather than keeping only apparently unchanged edits.
    json!({"changes": {
        "file:///project/src/App.vue": [{"range":{"start":{"line":3,"character":8},"end":{"line":3,"character":12}},"newText":"notify"}],
        "file:///project/src/useToast.ts": [{"range":{"start":{"line":1,"character":11},"end":{"line":1,"character":15}},"newText":"notify"}]
    }})
}

#[test]
fn root_and_dependency_edits_reject_the_complete_native_response() {
    for changed_uri in [
        "file:///project/src/App.vue",
        "file:///project/src/useToast.ts",
    ] {
        let (service, _socket) = build_lsp_service();
        let server = service.inner();
        let root = Url::parse("file:///project/src/App.vue").unwrap();
        let dependency = Url::parse("file:///project/src/useToast.ts").unwrap();
        server
            .state
            .documents
            .open(root.clone(), APP.into(), 1, "vue".into());
        server
            .state
            .documents
            .open(dependency.clone(), TOAST.into(), 1, "typescript".into());
        let (release, held) = oneshot::channel();
        let mut reply = Box::pin(server.native_request(async {
            held.await.unwrap();
            Ok(pending_native_reply())
        }));
        assert!(reply.as_mut().now_or_never().is_none());
        let changed = Url::parse(changed_uri).unwrap();
        server.state.documents.apply_changes(
            &changed,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "export const replacement = 1;".into(),
            }],
            2,
        );
        release.send(()).unwrap();
        assert_eq!(block_on(reply), Err(Error::content_modified()));
    }
}

#[test]
fn same_version_close_reopen_and_restored_content_do_not_authorize_old_edits() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let uri = Url::parse("file:///project/src/useToast.ts").unwrap();
    server
        .state
        .documents
        .open(uri.clone(), TOAST.into(), 1, "typescript".into());
    let (release, held) = oneshot::channel();
    let mut reply = Box::pin(server.native_request(async {
        held.await.unwrap();
        Ok(pending_native_reply())
    }));
    assert!(reply.as_mut().now_or_never().is_none());
    server.state.documents.close(&uri);
    server
        .state
        .documents
        .open(uri, TOAST.into(), 1, "typescript".into());
    release.send(()).unwrap();
    assert_eq!(block_on(reply), Err(Error::content_modified()));
}

#[test]
fn disk_config_and_workspace_changes_refuse_pending_native_results() {
    for change in 0..3 {
        let (service, _socket) = build_lsp_service();
        let server = service.inner();
        let (release, held) = oneshot::channel();
        let mut reply = Box::pin(server.native_request(async {
            held.await.unwrap();
            Ok(pending_native_reply())
        }));
        assert!(reply.as_mut().now_or_never().is_none());
        match change {
            0 => server.state.mark_corsa_disk_state_dirty(),
            1 => server
                .state
                .apply_lsp_initialization_options(Some(&json!({"typecheck":false}))),
            _ => server
                .state
                .set_workspace_root(std::path::PathBuf::from("/replacement")),
        }
        release.send(()).unwrap();
        assert_eq!(block_on(reply), Err(Error::content_modified()));
    }
}

#[test]
fn a_cancelled_waiter_never_opens_sources_and_the_next_transaction_reads_current_text() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let uri = Url::parse("file:///project/src/App.vue").unwrap();
    server
        .state
        .documents
        .open(uri.clone(), APP.into(), 1, "vue".into());
    let (release, held) = oneshot::channel();
    let mut first = Box::pin(server.native_request(async {
        held.await.unwrap();
        Ok(pending_native_reply())
    }));
    assert!(first.as_mut().now_or_never().is_none());
    let entered = std::sync::atomic::AtomicBool::new(false);
    let mut waiting = Box::pin(server.native_request(async {
        entered.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(pending_native_reply())
    }));
    assert!(waiting.as_mut().now_or_never().is_none());
    drop(waiting);
    server.state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<template>new revision</template>".into(),
        }],
        2,
    );
    release.send(()).unwrap();
    assert_eq!(block_on(first), Err(Error::content_modified()));
    assert!(!entered.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        block_on(server.native_request(async { Ok(server.state.documents.text(&uri)) })),
        Ok(Some("<template>new revision</template>".into()))
    );
}

#[test]
fn unchanged_sources_keep_the_complete_legacy_result() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    assert_eq!(
        block_on(server.native_request(async { Ok(pending_native_reply()) })),
        Ok(pending_native_reply())
    );
}

#[test]
fn ignored_older_versions_and_absent_closes_keep_the_complete_result() {
    let (service, _socket) = build_lsp_service();
    let server = service.inner();
    let uri = Url::parse("file:///project/src/App.vue").unwrap();
    server
        .state
        .documents
        .open(uri.clone(), APP.into(), 2, "vue".into());
    let (release, held) = oneshot::channel();
    let mut reply = Box::pin(server.native_request(async {
        held.await.unwrap();
        Ok(pending_native_reply())
    }));
    assert!(reply.as_mut().now_or_never().is_none());
    assert!(!server.state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "ignored old source".into(),
        }],
        1
    ));
    server
        .state
        .documents
        .close(&Url::parse("file:///project/src/absent.vue").unwrap());
    release.send(()).unwrap();
    assert_eq!(block_on(reply), Ok(pending_native_reply()));
}

#[test]
fn authored_revision_precedes_admission_but_valid_policy_reload_remains_usable() {
    #[derive(Clone, Copy)]
    enum Request {
        Native,
        Workspace,
        LocalCompletion,
    }
    const LOCAL_SOURCE: &str = "<script setup lang=\"ts\">\nconst palette = { tone: 'warm', count: 1 };\nconst chosen = palette.tone;\n</script>\n<template><div>{{ chosen }}</div></template>";
    for changed in [false, true] {
        for mode in [
            Request::Native,
            Request::Workspace,
            Request::LocalCompletion,
        ] {
            let project = crate::test_support::tempdir().unwrap();
            let root = vize_carton::path::canonicalize_non_verbatim(project.path());
            let config = root.join("vite.config.mjs");
            std::fs::write(&config, "export default {vue:{languageServer:{hover:false,typecheck:false,lint:false,ecosystem:false}}};").unwrap();
            let (service, _socket) = build_lsp_service();
            let server = service.inner();
            server.state.set_workspace_root(root.clone());
            server.state.load_workspace_config(&root);
            let uri = Url::from_file_path(root.join("App.vue")).unwrap();
            let source = if matches!(mode, Request::LocalCompletion) {
                LOCAL_SOURCE
            } else {
                APP
            };
            std::fs::write(root.join("App.vue"), source).unwrap();
            server
                .state
                .documents
                .open(uri.clone(), source.into(), 1, "vue".into());
            server.state.update_virtual_docs(&uri, source);
            std::fs::write(&config, "import {writeFileSync,existsSync} from 'node:fs'; writeFileSync(new URL('./entered',import.meta.url),''); while(!existsSync(new URL('./release',import.meta.url))) await new Promise(resolve=>setTimeout(resolve,1)); export default {vue:{languageServer:{hover:true,typecheck:false,lint:false,ecosystem:false}}};").unwrap();
            let mut operation = Box::pin(async {
                if matches!(mode, Request::LocalCompletion) {
                    let params = serde_json::from_value(json!({
                        "textDocument":{"uri":uri},
                        "position":{"line":2,"character":"const chosen = palette.".len()}
                    }))
                    .unwrap();
                    server
                        .completion(params)
                        .await
                        .map(|reply| serde_json::to_value(reply).unwrap())
                } else {
                    let view = server.for_document(&uri).await;
                    let entered = std::sync::atomic::AtomicBool::new(false);
                    let request = || async {
                        entered.store(true, std::sync::atomic::Ordering::SeqCst);
                        Ok(pending_native_reply())
                    };
                    let result = if matches!(mode, Request::Workspace) {
                        view.workspace_native_request(request).await
                    } else {
                        view.native_request(request()).await
                    };
                    assert!(entered.load(std::sync::atomic::Ordering::SeqCst));
                    result
                }
            });
            assert!(operation.as_mut().now_or_never().is_none());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !root.join("entered").is_file() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "actual config admission did not enter"
                );
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            if changed {
                let replacement = if matches!(mode, Request::LocalCompletion) {
                    vize_l0::cstr!(
                        "{source}\n<!-- a real unsaved edit, preserving the completion target -->\n"
                    )
                } else {
                    vize_l0::String::from("<template>new revision</template>")
                };
                assert!(server.state.documents.apply_changes(
                    &uri,
                    vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: replacement.as_str().into(),
                    }],
                    2
                ));
                server.state.update_virtual_docs(&uri, &replacement);
            }
            std::fs::write(root.join("release"), "").unwrap();
            let result = block_on(operation);
            let admitted = block_on(server.for_document_admitted(&uri));
            assert!(
                admitted.state.lsp_features().hover,
                "the actual valid replacement is admitted"
            );
            let expected_reply = if matches!(mode, Request::LocalCompletion) {
                let items = ["tone", "count"].map(|label| json!({
                    "label":label,"kind":10,"detail":"local object member",
                    "documentation":{"kind":"markdown","value":"Statically declared member of the local object `palette`."},
                    "sortText":vize_l0::cstr!("0{label}")
                }));
                // The edited buffer must still produce this exact Some answer.
                // Falling through to the native guard cannot qualify this case.
                let params = serde_json::from_value(json!({
                    "textDocument":{"uri":uri},
                    "position":{"line":2,"character":"const chosen = palette.".len()}
                }))
                .unwrap();
                let Some(Some(local)) = admitted.local_completion_response(&params).unwrap() else {
                    panic!("the actual local completion must retain both authored members");
                };
                assert_eq!(serde_json::to_value(local).unwrap(), json!(items));
                json!(items)
            } else {
                pending_native_reply()
            };
            if changed {
                assert_eq!(result, Err(Error::content_modified()));
            } else {
                assert_eq!(result, Ok(expected_reply));
            }
            assert_eq!(
                std::fs::read_to_string(root.join("App.vue")).unwrap(),
                source
            );
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "ArcWake counts wakeups through an Arc"
)]
fn an_edit_cancels_pending_requests_without_waiting_for_their_work() {
    struct Wake(std::sync::atomic::AtomicUsize);
    impl futures::task::ArcWake for Wake {
        fn wake_by_ref(this: &std::sync::Arc<Self>) {
            this.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    struct Dropped<'a>(&'a std::sync::atomic::AtomicBool);
    impl Drop for Dropped<'_> {
        fn drop(&mut self) {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
    for workspace in [false, true] {
        let (service, _socket) = build_lsp_service();
        let server = service.inner();
        let uri = Url::parse("file:///project/src/App.vue").unwrap();
        server
            .state
            .documents
            .open(uri.clone(), APP.into(), 1, "vue".into());
        let dropped = std::sync::atomic::AtomicBool::new(false);
        // The work never finishes: only the edit can end the request.
        let work = || async {
            let _guard = Dropped(&dropped);
            std::future::pending::<()>().await;
            Ok(pending_native_reply())
        };
        let mut reply: std::pin::Pin<Box<dyn std::future::Future<Output = _>>> = if workspace {
            Box::pin(server.workspace_native_request(work))
        } else {
            Box::pin(server.native_request(work()))
        };
        let wake = std::sync::Arc::new(Wake(std::sync::atomic::AtomicUsize::new(0)));
        let waker = futures::task::waker(wake.clone());
        let mut cx = std::task::Context::from_waker(&waker);
        assert!(reply.as_mut().poll(&mut cx).is_pending());
        assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));

        assert!(server.state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "<template>new revision</template>".into(),
            }],
            2,
        ));
        assert_eq!(wake.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(
            reply.as_mut().poll(&mut cx),
            std::task::Poll::Ready(Err(Error::content_modified()))
        );
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));

        // A request admitted after the edit answers normally.
        assert_eq!(
            block_on(server.native_request(async { Ok(server.state.documents.text(&uri)) })),
            Ok(Some("<template>new revision</template>".into()))
        );
    }
}
