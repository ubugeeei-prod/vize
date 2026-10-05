//! Real service requests distinguish proven builtin facts from unknown types.

use crate::{
    runtime::block_on,
    server::{MaestroServer, build_lsp_service},
};
use serde_json::{Value, json};
use tower::Service;
use tower_lsp::{LspService, jsonrpc::Request};

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/computed-inlay-hints/App.vue.txt"
);

fn original_builtin_hint() -> Value {
    json!({"position":{"line":3,"character":8},"label":": Ref<boolean>","kind":1,"tooltip":"Vue reactive binding (Ref)","paddingLeft":true})
}

fn send(service: &mut LspService<MaestroServer>, value: Value) -> Option<Value> {
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        service
            .call(serde_json::from_value::<Request>(value).unwrap())
            .await
            .unwrap()
            .map(|reply| serde_json::to_value(reply).unwrap())
    })
}

fn editor_only_document(source: &str) -> LspService<MaestroServer> {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let init = send(&mut service, json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"editor":true,"typecheck":false,"lint":false,"ecosystem":false}}})).unwrap();
    assert_eq!(init["result"]["capabilities"]["inlayHintProvider"], true);
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///Scorecard.vue","languageId":"vue","version":1,"text":source}}})
        ),
        None
    );
    service
}

fn query(service: &mut LspService<MaestroServer>, id: i32, range: Value) -> Value {
    send(service, json!({"jsonrpc":"2.0","id":id,"method":"textDocument/inlayHint","params":{"textDocument":{"uri":"file:///Scorecard.vue"},"range":range}})).unwrap()
}

fn builtin_hint(line: u32, character: u32, label: &str, wrapper: &str) -> Value {
    json!({"position":{"line":line,"character":character},"label":label,"kind":1,"tooltip":format!("Vue reactive binding ({wrapper})"),"paddingLeft":true})
}

#[test]
fn editor_only_original_scorecard_keeps_the_complete_three_hint_vector_and_ranges() {
    const SCORECARD: &str = include_str!(
        "../../../../../../tests/_fixtures/differential/lsp/computed-inlay-hints/Scorecard.vue.txt"
    );
    let mut service = editor_only_document(SCORECARD);
    let range = json!({"start":{"line":0,"character":0},"end":{"line":1000,"character":0}});
    assert_eq!(
        query(&mut service, 2, range.clone()),
        json!({"jsonrpc":"2.0","id":2,"result":[
            builtin_hint(5,11,": Ref<number>","Ref"),
            builtin_hint(6,13,": ComputedRef<number>","ComputedRef"),
            builtin_hint(7,13,": Ref<string>","Ref"),
        ]})
    );
    assert_eq!(
        query(
            &mut service,
            3,
            json!({"start":{"line":6,"character":13},"end":{"line":6,"character":13}})
        ),
        json!({"jsonrpc":"2.0","id":3,"result":[builtin_hint(6,13,": ComputedRef<number>","ComputedRef")]})
    );
    assert_eq!(
        query(
            &mut service,
            4,
            json!({"start":{"line":6,"character":14},"end":{"line":6,"character":14}})
        ),
        json!({"jsonrpc":"2.0","id":4,"result":null})
    );
    let changed = SCORECARD.replace("const count = ref(0)", "const count = readCount()");
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///Scorecard.vue","version":2},"contentChanges":[{"text":changed}]}})
        ),
        None
    );
    assert_eq!(
        query(&mut service, 5, range),
        json!({"jsonrpc":"2.0","id":5,"result":[builtin_hint(7,13,": Ref<string>","Ref")]})
    );
    let uri = tower_lsp::lsp_types::Url::parse("file:///Scorecard.vue").unwrap();
    assert_eq!(
        service.inner().state.documents.text(&uri).as_deref(),
        Some(changed.as_str())
    );
    #[cfg(feature = "native")]
    assert!(!service.inner().state.has_corsa_bridge());
}

#[test]
fn editor_only_builtin_facts_have_exact_utf16_crlf_coordinates_and_decline_unknown_types() {
    let source = "<script setup lang=\"ts\">\r\nimport { ref as r, computed as c } from 'vue';\r\nconst emoji = '🙂'; const café = r(0);\r\nconst doubled = c(() => café.value * 2);\r\nconst unknown = c(readProfile);\r\nconst cast = r(0 as number);\r\n</script>\r\n";
    let mut service = editor_only_document(source);
    let range = json!({"start":{"line":0,"character":0},"end":{"line":100,"character":0}});
    assert_eq!(
        query(&mut service, 2, range),
        json!({"jsonrpc":"2.0","id":2,"result":[
        builtin_hint(2,30,": Ref<number>","Ref"),
            builtin_hint(3,13,": ComputedRef<number>","ComputedRef"),
        ]})
    );
    let unknown = "<script setup lang=\"ts\">\nimport { ref, computed } from './fake';\nconst count = ref(0);\nconst doubled = computed(() => count.value * 2);\n</script>\n";
    let mut service = editor_only_document(unknown);
    assert_eq!(
        query(
            &mut service,
            2,
            json!({"start":{"line":0,"character":0},"end":{"line":100,"character":0}})
        ),
        json!({"jsonrpc":"2.0","id":2,"result":null})
    );
}

#[test]
fn disabled_checker_keeps_proven_builtin_facts_without_guessing_computed_types() {
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let init = send(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"typecheck":false,"lint":false,"inlayHints":true,"ecosystem":false}}})).unwrap();
    assert!(init["result"].is_object());
    let uri = "file:///App.vue";
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":ORIGINAL}}})
        ),
        None
    );
    let query = json!({"jsonrpc":"2.0","id":2,"method":"textDocument/inlayHint","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":100,"character":0}}}});
    assert_eq!(
        send(&mut service, query),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[original_builtin_hint()]}))
    );
    let uri = tower_lsp::lsp_types::Url::parse(uri).unwrap();
    assert_eq!(
        service.inner().state.documents.text(&uri).as_deref(),
        Some(ORIGINAL)
    );
    #[cfg(feature = "native")]
    assert!(!service.inner().state.has_corsa_bridge());
}

#[cfg(feature = "native")]
#[test]
fn unavailable_checker_keeps_known_builtin_without_rendering_computed_placeholders() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("vize.config.json"),
        serde_json::to_vec(
            &json!({"typeChecker":{"corsaPath":root.path().join("missing-native")}}),
        )
        .unwrap(),
    )
    .unwrap();
    let (mut service, socket) = build_lsp_service();
    drop(socket);
    let root_uri = tower_lsp::lsp_types::Url::from_directory_path(root.path()).unwrap();
    let uri = root_uri.join("App.vue").unwrap();
    let init = send(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":root_uri,"capabilities":{},"initializationOptions":{"typecheck":true,"lint":false,"inlayHints":true,"ecosystem":false}}})).unwrap();
    assert!(init["result"].is_object());
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":ORIGINAL}}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":2,"method":"textDocument/inlayHint","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":11,"character":0}}}})
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[original_builtin_hint()]}))
    );
    assert!(service.inner().state.corsa_init_failure().is_some());
    assert_eq!(
        service.inner().state.documents.text(&uri).as_deref(),
        Some(ORIGINAL)
    );
}
