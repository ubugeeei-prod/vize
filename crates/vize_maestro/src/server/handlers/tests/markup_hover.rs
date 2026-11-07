//! Rich authored markup hover must not enter type-project routing.

use tower_lsp::{
    LanguageServer, LspService,
    lsp_types::{
        HoverContents, HoverParams, Position, TextDocumentIdentifier, TextDocumentPositionParams,
        Url,
    },
};

use super::super::MaestroServer;

#[test]
fn resident_markup_hover_does_not_wait_for_the_native_transaction() {
    crate::runtime::block_on(async {
        let project = crate::test_support::tempdir().unwrap();
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({
                "hover": true,
                "typecheck": true,
                "lint": false,
                "ecosystem": false
            })));
        let source = "<template><button @click=\"save\">Save</button></template>";
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        std::fs::write(uri.to_file_path().unwrap(), source).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        server.state.update_virtual_docs(&uri, source);
        server.state.set_corsa_disk_watcher_registered();

        let offset = source.find("@click").unwrap() + 2;
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position::new(line, character),
            },
            work_done_progress_params: Default::default(),
        };

        let _native = server.state.corsa_request_scope().await;
        let hover =
            crate::runtime::timeout(std::time::Duration::from_secs(3), server.hover(params))
                .await
                .expect("resident markup must bypass a held native transaction")
                .unwrap()
                .expect("resident directive hover");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("directive hover must stay rich Markdown");
        };
        assert!(markup.value.contains("Vue event listener"));
        assert_eq!(server.state.canonical_bridge_request_count(), 0);
    });
}

#[test]
fn typecheck_sessions_keep_native_markup_under_the_live_backend() {
    let project = crate::test_support::tempdir().unwrap();
    let state = crate::server::ServerState::new();
    let source = "<template><label :for=\"id\" @click=\"save\">Name</label></template>";
    let uri = Url::from_file_path(project.path().join("Typed.vue")).unwrap();
    std::fs::write(uri.to_file_path().unwrap(), source).unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, source);
    assert!(state.is_lsp_typecheck_enabled());

    for needle in ["label", "for"] {
        let offset = source.find(needle).unwrap() + 1;
        let ctx = crate::ide::IdeContext::new(&state, &uri, offset).unwrap();
        assert!(
            crate::ide::HoverService::hover_resident_markup(&ctx).is_none(),
            "{needle} must retain the exact DOM-library type"
        );
    }

    let offset = source.find("@click").unwrap() + 2;
    let ctx = crate::ide::IdeContext::new(&state, &uri, offset).unwrap();
    assert!(crate::ide::HoverService::hover_resident_markup(&ctx).is_some());
}

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
                r#"**<button>**

_HTML element_

```typescript
const element: HTMLElementTagNameMap["button"]
```

Provides an interactive button that can trigger an action or submit a form.

**Editor behavior**
- Go-to-definition uses TypeScript DOM lib data when the type service is available.
- Component resolution is skipped for this tag because it is part of the platform DOM surface.

**Example**

```vue
<template>
  <button>...</button>
</template>
```

**Docs**

[MDN reference](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/button)

[HTML Living Standard](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)"#,
            ),
            (
                "disabled",
                r#"**disabled on <button>**

_HTML attribute_

```typescript
disabled: HTMLElementTagNameMap["button"]["disabled"]
```

Native DOM attribute recognized by the Vue template compiler. Vue patches it on the platform element rather than resolving it as a component prop.

**Runtime surface**

Boolean HTML attribute

**Editor behavior**
- Hover and go-to-definition use the DOM library type for the reflected property when the type service is available.
- Component prop lookup is skipped because the enclosing tag is a native element.

**Example**

```vue
<template>
  <button disabled>...</button>
</template>
```

**Docs**

[MDN reference](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/disabled)"#,
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
            assert_eq!(markup.value, expected);
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
        assert_eq!(
            markup.value,
            r#"**@click**

_Vue event listener_

**Example**

```vue
@click="handler"
```

Attaches a DOM or component event listener. The handler expression is evaluated in component scope.

**Template behavior**
- `$event` is available inside inline handler expressions.
- Event modifiers such as `.stop`, `.prevent`, and key modifiers are compiled by Vue.

**Docs**

[Vue Event Handling](https://vuejs.org/guide/essentials/event-handling.html)"#
        );

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
