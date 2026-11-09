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
use crate::virtual_code::{VirtualDocument, VirtualDocuments, project_template_fragment};

fn source(color: &str) -> String {
    vize_l0::cstr!(
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
        vize_l0::cstr!("\n.box {{ color: {color}; }}\n").to_string(),
    )]
}

fn legacy_standalone_document(uri: &Url, source: &str) -> VirtualDocument {
    let allocator = vize_l0::Allocator::new();
    let (root, _errors) = vize_armature::parse(&allocator, source);
    let base_uri = uri.path();
    project_template_fragment(
        None,
        false,
        0,
        None,
        &root,
        0,
        vize_l0::cstr!("{base_uri}.__template.ts").to_string(),
    )
}

#[cfg(feature = "retained-program")]
fn assert_petite_semantic_reuse_is_selected(source: &str) {
    let allocator = vize_l0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty(), "legacy fixture must parse: {errors:?}");
    assert!(
        super::petite_semantic_reuse(&allocator, &root, source).is_some(),
        "admitted petite source must select the Levels reuse branch"
    );
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

#[test]
#[cfg(feature = "retained-program")]
fn standalone_petite_level_route_preserves_the_complete_legacy_projection() {
    let state = ServerState::new();
    state.set_dialect_config(Some(vize_l0::dialect::VueDialect::PetiteVue));
    for (index, source) in [
        r#"<div v-scope="{ count: 0, msg: 'x' }" v-effect="count > 0">{{ count }} {{ msg }}</div>"#,
        r#"<main v-scope><i v-scope="count"></i><b v-effect=""></b></main>"#,
        r#"<section v-scope="({ 'label': '&amp;', total: 1 })"><button v-effect="total++"></button></section>"#,
        r#"<div v-scope="{ first: '&amp;', second: 2 }"><span>{{ second }}</span></div>"#,
        r#"<div v-scope="{ count: 0 }" v-effect="count++; count += 1"></div>"#,
        r#"<svg><foreignObject><article v-scope="{ item: 1 }"><i v-effect="item"></i></article></foreignObject></svg>"#,
        r#"<div><p>é</p><div v-scope="{ msg: 'é', café: 1, count: 0 }" v-effect="count + café">{{ count }}</div></div>"#,
    ]
    .into_iter()
    .enumerate()
    {
        assert_petite_semantic_reuse_is_selected(source);
        let uri = Url::parse(&vize_l0::cstr!("file:///petite-{index}.html")).unwrap();
        let legacy = legacy_standalone_document(&uri, source);
        state.update_virtual_docs(&uri, source);
        let actual = state
            .get_virtual_docs(&uri)
            .and_then(|documents| documents.template.clone())
            .expect("standalone template document");
        assert_eq!(
            vize_l0::cstr!("{actual:#?}"),
            vize_l0::cstr!("{legacy:#?}"),
            "source: {source}"
        );
    }
}

#[test]
#[cfg(not(feature = "retained-program"))]
fn standalone_petite_without_retained_program_keeps_the_complete_legacy_projection() {
    let state = ServerState::new();
    state.set_dialect_config(Some(vize_l0::dialect::VueDialect::PetiteVue));
    let uri = Url::parse("file:///petite-without-retained-program.html").unwrap();
    let source = r#"<div v-scope="{ count: 0 }" v-effect="count++">{{ count }}</div>"#;
    let legacy = legacy_standalone_document(&uri, source);

    state.update_virtual_docs(&uri, source);
    let actual = state
        .get_virtual_docs(&uri)
        .and_then(|documents| documents.template.clone())
        .expect("fallback template document");
    assert_eq!(vize_l0::cstr!("{actual:#?}"), vize_l0::cstr!("{legacy:#?}"));
}

#[test]
fn refused_petite_document_keeps_the_complete_legacy_projection() {
    let state = ServerState::new();
    state.set_dialect_config(Some(vize_l0::dialect::VueDialect::PetiteVue));
    let uri = Url::parse("file:///petite-refused.html").unwrap();
    let source = "<div v-scope='unterminated";
    let legacy = legacy_standalone_document(&uri, source);

    #[cfg(feature = "retained-program")]
    {
        let allocator = vize_l0::Allocator::new();
        let (root, _errors) = vize_armature::parse(&allocator, source);
        assert!(super::petite_semantic_reuse(&allocator, &root, source).is_none());
    }

    state.update_virtual_docs(&uri, source);
    let actual = state
        .get_virtual_docs(&uri)
        .and_then(|documents| documents.template.clone())
        .expect("fallback template document");
    assert_eq!(vize_l0::cstr!("{actual:#?}"), vize_l0::cstr!("{legacy:#?}"));
}
