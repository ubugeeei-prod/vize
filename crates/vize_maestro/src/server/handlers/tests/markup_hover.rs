//! Rich authored markup hover must not enter type-project routing.

use tower_lsp::{
    LspService,
    lsp_types::{
        HoverContents, HoverParams, Position, TextDocumentIdentifier, TextDocumentPositionParams,
        Url,
    },
};

use super::super::MaestroServer;

#[test]
fn native_markup_hovers_are_rich_without_entering_corsa_routing() {
    crate::runtime::block_on(async {
        let project = crate::test_support::tempdir().unwrap();
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({
                "hover": true,
                "typecheck": false,
                "lint": false,
                "ecosystem": false
            })));
        let source = "<template>\n  <button disabled @click=\"save\">Save</button>\n  <Widget :count=\"n\" @save=\"save\" v-model=\"model\" />\n</template>\n";
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        std::fs::write(uri.to_file_path().unwrap(), source).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        server.state.update_virtual_docs(&uri, source);
        let routed = server.for_document(&uri).await;
        assert!(!routed.state.is_lsp_typecheck_enabled());
        assert!(routed.state.lsp_features().hover);

        for (needle, expected) in [
            (
                "button",
                [
                    "**<button>**",
                    "_HTML element_",
                    "HTMLElementTagNameMap[\"button\"]",
                    "Provides an interactive button",
                    "**Editor behavior**",
                    "**Example**",
                    "MDN reference",
                    "HTML Living Standard",
                ]
                .as_slice(),
            ),
            (
                "disabled",
                [
                    "**disabled on <button>**",
                    "_HTML attribute_",
                    "HTMLElementTagNameMap[\"button\"][\"disabled\"]",
                    "**Runtime surface**",
                    "Boolean HTML attribute",
                    "**Editor behavior**",
                    "**Example**",
                    "MDN reference",
                ]
                .as_slice(),
            ),
        ] {
            let offset = source.find(needle).unwrap() + 1;
            let (line, character) = crate::ide::offset_to_position(source, offset);
            let hover = routed
                .hover_request(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        position: Position::new(line, character),
                    },
                    work_done_progress_params: Default::default(),
                })
                .await
                .unwrap()
                .expect("resident markup hover");
            let HoverContents::Markup(markup) = hover.contents else {
                panic!("markup hover must stay rich Markdown");
            };
            let range = hover.range.expect("authored markup range");
            assert_eq!(range.start, Position::new(line, character - 1));
            let (_, end_character) =
                crate::ide::offset_to_position(source, source.find(needle).unwrap() + needle.len());
            assert_eq!(range.end, Position::new(line, end_character));
            for part in expected {
                assert!(
                    markup.value.contains(part),
                    "missing {part:?}: {}",
                    markup.value
                );
            }
        }

        let offset = source.find("@click").unwrap() + 2;
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let hover = routed
            .hover_request(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position: Position::new(line, character),
                },
                work_done_progress_params: Default::default(),
            })
            .await
            .unwrap()
            .expect("resident Vue directive hover");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("directive hover must stay rich Markdown");
        };
        for part in [
            "**@click**",
            "_Vue event listener_",
            "**Template behavior**",
            "Vue Event Handling",
        ] {
            assert!(
                markup.value.contains(part),
                "missing {part:?}: {}",
                markup.value
            );
        }

        assert_eq!(routed.state.canonical_bridge_request_count(), 0);
        assert_eq!(routed.state.corsa_init_failure(), None);
        for (needle, inside, expected_requests) in
            [(":count", 2, 1), ("@save", 2, 2), ("v-model", 2, 3)]
        {
            let offset = source.find(needle).unwrap() + inside;
            let (line, character) = crate::ide::offset_to_position(source, offset);
            let _ = routed
                .hover_request(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        position: Position::new(line, character),
                    },
                    work_done_progress_params: Default::default(),
                })
                .await;
            // Component props, events, and models retain the typed route.
            assert_eq!(
                routed.state.canonical_bridge_request_count(),
                expected_requests
            );
        }
    });
}
