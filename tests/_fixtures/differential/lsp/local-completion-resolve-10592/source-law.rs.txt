//! Local completion answers must not borrow a retired native project owner.

use super::*;
use serde_json::{Value, json};
use tower_lsp::lsp_types::{CompletionItem, Documentation};

fn item() -> CompletionItem {
    serde_json::from_value(json!({
        "label":"id","kind":10,"detail":"Vue Router file route param: string",
        "sortText":"0","filterText":"id","insertText":"${1:id}",
        "insertTextFormat":2,"insertTextMode":1,"preselect":true,
        "documentation":{"kind":"markdown","value":"immutable local documentation"},
        "labelDetails":{"detail":" local","description":"catalog"},
        "tags":[1],"commitCharacters":["."],
        "textEdit":{"range":{"start":{"line":3,"character":4},"end":{"line":3,"character":6}},"newText":"id"},
        "additionalTextEdits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"// 雪😀\n"}],
        "command":{"title":"local","command":"fixture.local","arguments":[{"opaque":true}]}
    })).unwrap()
}

fn retired_service() -> LspService<MaestroServer> {
    let (service, socket) = LspService::new(MaestroServer::new);
    drop(socket);
    service.inner().state.retire_project_contexts();
    assert!(service.inner().state.project_context_retired());
    service
}

fn assert_unstarted(server: &MaestroServer) {
    assert!(!server.state.has_corsa_bridge());
    assert!(server.state.corsa_init_failure().is_none());
}

#[test]
fn unbound_item_preserves_every_field_after_primary_retirement() {
    crate::runtime::block_on(async {
        let service = retired_service();
        let server = service.inner();
        let item = item();
        let result = server.completion_resolve(item.clone()).await.unwrap();
        assert_eq!(result, item);
        assert_unstarted(server);
    });
}

#[test]
fn css_catalog_resolution_keeps_all_insertion_fields_and_never_opens_a_native_owner() {
    crate::runtime::block_on(async {
        let service = retired_service();
        let server = service.inner();
        let mut item = item();
        item.label = "display".into();
        item.documentation = None;
        item.data = Some(json!({"vizeCss":{"kind":"property","name":"display","property":null}}));
        let result = server.completion_resolve(item.clone()).await.unwrap();
        let Some(Documentation::MarkupContent(documentation)) = &result.documentation else {
            panic!("existing rich CSS catalog documentation required");
        };
        assert!(documentation.value.contains("**Syntax**"));
        let mut insertion = result;
        insertion.documentation = None;
        assert_eq!(insertion, item);
        assert_unstarted(server);
    });
}

#[test]
fn invalid_css_catalog_identity_and_mismatched_label_or_kind_remain_unchanged() {
    crate::runtime::block_on(async {
        let service = retired_service();
        let server = service.inner();
        for (label, kind, payload) in [
            ("display", 10, Value::Null),
            ("display", 10, json!({})),
            (
                "display",
                10,
                json!({"kind":"property","name":"missing","property":null}),
            ),
            (
                "foreign",
                10,
                json!({"kind":"property","name":"display","property":null}),
            ),
            (
                "display",
                3,
                json!({"kind":"property","name":"display","property":null}),
            ),
        ] {
            let mut item = item();
            item.label = label.into();
            item.kind = Some(serde_json::from_value(json!(kind)).unwrap());
            item.data = Some(json!({"vizeCss":payload}));
            assert_eq!(server.completion_resolve(item.clone()).await.unwrap(), item);
        }
        assert_unstarted(server);
    });
}

#[test]
fn native_marker_presence_and_unknown_data_keep_the_original_freshness_boundary() {
    crate::runtime::block_on(async {
        let service = retired_service();
        let server = service.inner();
        for data in [
            Value::Null,
            json!({}),
            json!({"unknownOwner":{"uri":"file:///unavailable.vue"}}),
            json!({"vizeCompletion":null}),
            json!({"vizeCompletion":{}}),
            json!({"vizeCompletion":{"uri":false}}),
            json!({"vizeCompletion":{"uri":"not a URI"}}),
            json!({"vizeCompletion":{"uri":"file:///unavailable.vue","revision":1}}),
            json!({"vizeCompletion":null,"vizeCss":{"kind":"property","name":"display","property":null}}),
        ] {
            let mut item = item();
            item.data = Some(data.clone());
            assert_eq!(
                server.completion_resolve(item).await.unwrap_err().code,
                tower_lsp::jsonrpc::ErrorCode::ContentModified,
                "native/unknown ownership must not become a local answer: {data}"
            );
        }
        assert_unstarted(server);
    });
}
