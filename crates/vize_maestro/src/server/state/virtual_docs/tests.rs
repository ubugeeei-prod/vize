//! Lock-discipline regressions for the virtual-document cache (#3377).
//!
//! Both tests below deadlock instead of failing on the pre-fix code, because
//! that is exactly what the defect does to a live `vize lsp` session: the
//! reader holding a `DashMap` shard guard can only release it by being polled
//! on the very thread that is already parked in `parking_lot` waiting for the
//! shard's write lock. A hung test is the honest reproduction of a hung server.

use tower_lsp::lsp_types::Url;

use crate::ide::IdeContext;
use crate::server::ServerState;
use crate::virtual_code::VirtualDocuments;

fn source(color: &str) -> String {
    vize_s0::cstr!(
        "<script setup lang=\"ts\">\nconst message = 'hi'\n</script>\n\
         <template>\n  <div>{{{{ message }}}}</div>\n</template>\n\
         <style scoped>\n.box {{ color: {color}; }}\n</style>\n"
    )
    .to_string()
}

/// Every style virtual document as `(uri, content)`, so assertions pin the
/// whole generated set rather than one probed field.
fn styles(docs: &VirtualDocuments) -> Vec<(String, String)> {
    docs.styles
        .iter()
        .map(|style| (style.uri.clone(), style.content.clone()))
        .collect()
}

fn style_documents(color: &str) -> Vec<(String, String)> {
    vec![(
        "/Snapshot.vue.__style_0.css".to_string(),
        vize_s0::cstr!("\n.box {{ color: {color}; }}\n").to_string(),
    )]
}

/// `get_virtual_docs` must return an owned snapshot, never a shard guard, so a
/// snapshot that is still alive cannot block a concurrent cache write.
#[test]
fn get_virtual_docs_snapshot_holds_no_shard_lock() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Snapshot.vue").unwrap();
    state.update_virtual_docs(&uri, &source("red"));

    let snapshot = state.get_virtual_docs(&uri).unwrap();
    state.update_virtual_docs(&uri, &source("blue"));

    assert_eq!(styles(&snapshot), style_documents("red"));
    assert_eq!(
        styles(&state.get_virtual_docs(&uri).unwrap()),
        style_documents("blue")
    );
    assert!(
        state
            .get_virtual_docs(&Url::parse("file:///Absent.vue").unwrap())
            .is_none()
    );
}

/// The shape the LSP actually runs: `&IdeContext` stays alive across an
/// `.await` (hover, completion, definition, references, rename all do this),
/// and a `didChange` polled on the same executor thread writes the cache while
/// that context is suspended.
#[test]
fn ide_context_survives_a_virtual_docs_write_across_an_await() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Snapshot.vue").unwrap();
    let before = source("red");
    state
        .documents
        .open(uri.clone(), before.clone(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, &before);

    let observed = crate::runtime::block_on(async {
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        // Stands in for `get_corsa_bridge().await` handing the single executor
        // thread to a queued `didChange` while `ctx` is still live.
        futures::future::ready(()).await;
        state.update_virtual_docs(&uri, &source("blue"));
        futures::future::ready(()).await;
        styles(ctx.virtual_docs.as_ref().unwrap())
    });

    assert_eq!(observed, style_documents("red"));
    assert_eq!(
        styles(&state.get_virtual_docs(&uri).unwrap()),
        style_documents("blue")
    );
}
