use super::*;
use crate::server::ServerState;

fn owner() -> Arc<CorsaBridge> {
    Arc::new(CorsaBridge::new())
}

#[test]
fn current_request_takes_its_receipt_once_without_native_startup() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, "<template />".into());
    let bridge = owner();
    let receipt = InitialCanonicalDocument::new(
        &ctx,
        &bridge,
        "whole opaque payload",
        state.corsa_request_stamp(),
    );
    assert_eq!(receipt.take(&ctx, &bridge), Some("whole opaque payload"));
    assert_eq!(receipt.take(&ctx, &bridge), None);
    assert!(!bridge.is_initialized());
}

#[test]
fn foreign_bridge_and_equal_text_from_another_arc_cannot_consume_receipt() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, "<template />".into());
    let bridge = owner();
    let foreign = owner();
    let receipt = InitialCanonicalDocument::new(&ctx, &bridge, 42, state.corsa_request_stamp());
    assert_eq!(receipt.take(&ctx, &foreign), None);
    let copied = IdeContext::for_unopened(&state, &uri, 0, ctx.content.as_str().to_owned());
    assert_eq!(&*copied.content, &*ctx.content);
    assert!(!Arc::ptr_eq(&copied.content, &ctx.content));
    assert_eq!(receipt.take(&copied, &bridge), None);
    assert_eq!(receipt.take(&ctx, &bridge), Some(42));
}

#[test]
fn another_document_revision_and_close_reopen_refuse_old_receipts() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let other = Url::parse("file:///workspace/Other.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, "<template />".into());
    let bridge = owner();
    let receipt = InitialCanonicalDocument::new(&ctx, &bridge, 42, state.corsa_request_stamp());
    state
        .documents
        .open(other, "<template />".into(), 0, "vue".into());
    assert!(Arc::ptr_eq(
        &state.documents.text_snapshot(&uri).unwrap(),
        &ctx.content
    ));
    assert_eq!(receipt.take(&ctx, &bridge), None);
    let reopened = InitialCanonicalDocument::new(&ctx, &bridge, 43, state.corsa_request_stamp());
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), "<template />".into(), 0, "vue".into());
    let current = IdeContext::new(&state, &uri, 0).unwrap();
    assert_eq!(&*ctx.content, &*current.content);
    assert!(!Arc::ptr_eq(&ctx.content, &current.content));
    assert_eq!(reopened.take(&current, &bridge), None);
}

#[test]
fn environment_retirement_refuses_receipt_without_clearing_the_payload_early() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, "<template />".into());
    let bridge = owner();
    let receipt = InitialCanonicalDocument::new(&ctx, &bridge, 42, state.corsa_request_stamp());
    state.retire_project_contexts();
    assert_eq!(receipt.take(&ctx, &bridge), None);
    assert_eq!(*receipt.receipt.lock().unwrap(), Some(42));
}
