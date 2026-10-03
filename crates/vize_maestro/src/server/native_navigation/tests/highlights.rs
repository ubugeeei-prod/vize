mod lifecycle;

use super::{Value, json, send, service};

const URI: &str = "file:///native-highlight.vue";
fn open(service: &mut tower_lsp::LspService<super::MaestroServer>, source: &str, language: &str) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":URI,"languageId":language,"version":1,"text":source}}})
        ),
        None
    );
}
fn request(id: i32, method: &str, at: (u32, u32)) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":URI},"position":{"line":at.0,"character":at.1}}})
}
fn position(source: &str, byte: usize) -> (u32, u32) {
    let prefix = source.get(..byte).unwrap();
    (
        prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn occurrence(source: &str, name: &str, index: usize, kind: u32) -> ((u32, u32), Value) {
    let byte = source.match_indices(name).nth(index).unwrap().0;
    let start = position(source, byte);
    let end = position(source, byte + name.len());
    (
        start,
        json!({"range":{"start":{"line":start.0,"character":start.1},"end":{"line":end.0,"character":end.1}},"kind":kind}),
    )
}
fn error(id: i32, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

#[test]
fn whole_original_js_ts_and_react_highlight_arrays_keep_actual_site_kinds() {
    for language in ["javascript", "typescript"] {
        let mut service = service();
        let source = "let value=1;value;value=2;value+=3;value++;";
        open(&mut service, source, language);
        let (at, first) = occurrence(source, "value", 0, 1);
        let wanted = vec![
            first,
            occurrence(source, "value", 1, 2).1,
            occurrence(source, "value", 2, 3).1,
            occurrence(source, "value", 3, 3).1,
            occurrence(source, "value", 4, 3).1,
        ];
        assert_eq!(
            send(&mut service, request(2, "vize/nativeDocumentHighlight", at)),
            Some(json!({"jsonrpc":"2.0","id":2,"result":wanted}))
        );
    }
    for language in ["javascriptreact", "typescriptreact"] {
        let mut service = service();
        let source = "const Widget=1;let value=1;const view=<Widget value={value}/>;value++;";
        open(&mut service, source, language);
        let (at, component) = occurrence(source, "Widget", 1, 2);
        assert_eq!(
            send(&mut service, request(2, "vize/nativeDocumentHighlight", at)),
            Some(
                json!({"jsonrpc":"2.0","id":2,"result":[occurrence(source,"Widget",0,1).1,component]})
            )
        );
        assert_eq!(
            send(
                &mut service,
                request(
                    3,
                    "vize/nativeDocumentHighlight",
                    occurrence(source, "value", 1, 2).0
                )
            ),
            Some(json!({"jsonrpc":"2.0","id":3,"result":[]}))
        );
    }
}

#[test]
fn whole_selected_var_declarations_and_implicit_event_keep_exact_arrays() {
    let mut service = service();
    let source = "<template><button @click='var value=$event;{var value=2;}value=3;value+=1;return value;$event' @blur='$event'/></template>";
    open(&mut service, source, "vue");
    let (at, last) = occurrence(source, "value", 4, 2);
    let wanted = vec![
        occurrence(source, "value", 0, 1).1,
        occurrence(source, "value", 1, 1).1,
        occurrence(source, "value", 2, 3).1,
        occurrence(source, "value", 3, 3).1,
        last,
    ];
    assert_eq!(
        send(
            &mut service,
            request(2, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":wanted}))
    );
    let (at, event) = occurrence(source, "$event", 0, 2);
    assert_eq!(
        send(
            &mut service,
            request(3, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[event,occurrence(source,"$event",1,2).1]}))
    );
}

#[test]
fn whole_original_and_selected_entity_utf16_profiles_preserve_typed_refusals() {
    let mut service = service();
    let source = "<script setup>/*kept\r\n😀*/ let café=1;café=2;café++;</script><template>{{caf&#233;}}</template>";
    open(&mut service, source, "vue");
    let (at, read) = occurrence(source, "caf&#233;", 0, 2);
    assert_eq!(
        send(&mut service, request(2, "vize/nativeDocumentHighlight", at)),
        Some(
            json!({"jsonrpc":"2.0","id":2,"result":[occurrence(source,"café",0,1).1,occurrence(source,"café",1,3).1,occurrence(source,"café",2,3).1,read]})
        )
    );
    assert_eq!(
        send(
            &mut service,
            request(3, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(error(
            3,
            -32011,
            "Native original selected SFC observation refused"
        ))
    );
    assert_eq!(
        send(
            &mut service,
            request(4, "vize/nativeDocumentHighlight", (1, 1))
        ),
        Some(error(4, -32602, "Invalid native UTF-16 position"))
    );
    let selected = "<template><button @click='/*kept\r\n😀*/ let café=$event;caf&#233;=1;café++;return café'/></template>";
    open(&mut service, selected, "vue");
    let (at, write) = occurrence(selected, "caf&#233;", 0, 3);
    assert_eq!(
        send(
            &mut service,
            request(5, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(
            json!({"jsonrpc":"2.0","id":5,"result":[occurrence(selected,"café",0,1).1,write,occurrence(selected,"café",1,3).1,occurrence(selected,"café",2,2).1]})
        )
    );
    assert_eq!(
        send(
            &mut service,
            request(6, "vize/nativeTemplateDocumentHighlight", (1, 1))
        ),
        Some(error(6, -32602, "Invalid native UTF-16 position"))
    );
}

#[test]
fn actual_change_and_close_shift_owned_highlight_ranges_and_refuse_unavailable_document() {
    let mut service = service();
    let source = "<template><button @click='let value=$event;return value'/></template>";
    open(&mut service, source, "vue");
    let (at, first) = occurrence(source, "value", 0, 1);
    assert_eq!(
        send(
            &mut service,
            request(2, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[first,occurrence(source,"value",1,2).1]}))
    );
    let changed =
        "<template>\n<button @click='let freshName=$event;return freshName'/>\n</template>";
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":changed}]}})
        ),
        None
    );
    let (at, first) = occurrence(changed, "freshName", 0, 1);
    assert_eq!(
        send(
            &mut service,
            request(3, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(
            json!({"jsonrpc":"2.0","id":3,"result":[first,occurrence(changed,"freshName",1,2).1]})
        )
    );
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":URI}}})
        ),
        None
    );
    assert_eq!(
        send(
            &mut service,
            request(4, "vize/nativeTemplateDocumentHighlight", at)
        ),
        Some(error(4, -32602, "Native document unavailable"))
    );
}
