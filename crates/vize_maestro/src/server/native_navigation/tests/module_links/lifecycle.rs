//! Original host mutation and cancellation use the production RPC dispatcher.
use super::super::{Service, block_on, request};
use super::{error, initialized, links, open_at, send};
use serde_json::json;
use std::{future::Future, task::Context};
use tower_lsp::lsp_types::Url;

#[test]
fn module_link_rpc_real_close_and_equal_version_reopen_never_return_missing_document_as_empty() {
    let uri = Url::parse("file:///original.ts").unwrap();
    let mut service = initialized(None);
    open_at(&mut service, &uri, "const local=1;local;", "javascript");
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(error(2, -32015, "Native module-link context unavailable"))
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose",
        "params":{"textDocument":{"uri":uri}}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, links(3, &uri)),
        Some(error(3, -32602, "Native document unavailable"))
    );
    open_at(&mut service, &uri, "const local=1;local;", "javascript");
    assert_eq!(
        send(&mut service, links(4, &uri)),
        Some(error(4, -32015, "Native module-link context unavailable"))
    );
}

#[test]
fn module_link_rpc_wire_cancellation_never_publishes_original_operand_or_context_error() {
    let uri = Url::parse("file:///original.ts").unwrap();
    let mut service = initialized(None);
    open_at(&mut service, &uri, "import './child.ts';", "typescript");
    let resume = service
        .inner()
        .navigation
        .as_ref()
        .unwrap()
        .pause_module_link_worker(&uri);
    let mut pending = Box::pin(service.call(request(links(9, &uri))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":9}})
        ),
        None
    );
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Canceled")
    );
    resume.send(()).unwrap();
    assert_eq!(
        send(&mut service, links(10, &uri)),
        Some(error(10, -32015, "Native module-link context unavailable"))
    );
}

#[test]
fn module_link_rpc_original_host_edit_cancels_pending_reply_before_unavailable_context() {
    let uri = Url::parse("file:///original.ts").unwrap();
    let mut service = initialized(None);
    open_at(&mut service, &uri, "import './child.ts';", "typescript");
    let resume = service
        .inner()
        .navigation
        .as_ref()
        .unwrap()
        .pause_module_link_worker(&uri);
    let mut pending = Box::pin(service.call(request(links(9, &uri))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    open_at(&mut service, &uri, "const actual=2;actual;", "javascript");
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32800, "Native query cancelled")
    );
    resume.send(()).unwrap();
    assert_eq!(
        send(&mut service, links(10, &uri)),
        Some(error(10, -32015, "Native module-link context unavailable"))
    );
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_rpc_repeated_actual_file_events_and_target_changes_keep_complete_point_responses() {
    use super::{files, link};
    let (_dir, root, uri) = files();
    let target = Url::from_file_path(root.join("child.ts")).unwrap();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    open_at(&mut service, &uri, "import './child.ts';", "typescript");
    let expected = |id| json!({"jsonrpc":"2.0","id":id,"result":[link(0,7,19,target.clone())]});
    assert_eq!(send(&mut service, links(2, &uri)), Some(expected(2)));
    std::fs::remove_file(root.join("child.ts")).unwrap();
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles",
        "params":{"changes":[{"uri":target,"type":3}]}})
        ),
        None
    );
    assert_eq!(
        send(&mut service, links(3, &uri)),
        Some(error(3, -32018, "Native module target unavailable"))
    );
    std::fs::write(root.join("child.ts"), "export const actual=3;").unwrap();
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"workspace/didCreateFiles",
        "params":{"files":[{"uri":target}]}})
        ),
        None
    );
    assert_eq!(send(&mut service, links(4, &uri)), Some(expected(4)));
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles",
        "params":{"changes":[]}})
        ),
        None
    );
    assert_eq!(send(&mut service, links(5, &uri)), Some(expected(5)));
}
