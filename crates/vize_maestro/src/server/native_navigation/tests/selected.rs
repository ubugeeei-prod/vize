use super::{Value, json, send, service};

const URI: &str = "file:///selected.vue";

fn open(service: &mut tower_lsp::LspService<super::MaestroServer>, source: &str) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":URI,"languageId":"vue","version":1,"text":source}}})
        ),
        None
    );
}
fn request(id: i32, method: &str, line: u32, character: u32, include: bool) -> Value {
    if method == "vize/nativeTemplateReferences" {
        json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character},"context":{"includeDeclaration":include}}})
    } else {
        json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character}}})
    }
}
fn location(line: u32, start: u32, end: u32) -> Value {
    json!({"uri":URI,"range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}}})
}
fn position(source: &str, byte: usize) -> (u32, u32) {
    let prefix = &source[..byte];
    (
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn occurrence(source: &str, needle: &str, index: usize) -> ((u32, u32), Value) {
    let byte = source.match_indices(needle).nth(index).unwrap().0;
    let (line, column) = position(source, byte);
    let (_, end) = position(source, byte + needle.len());
    ((line, column), location(line, column, end))
}
fn error(id: i32, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

#[test]
fn whole_selected_definition_and_reference_envelopes_preserve_authored_utf16_entities() {
    let mut service = service();
    let source =
        "<template><button @click='/*kept\r\n😀*/ let café=$event; return caf&#233;'/></template>";
    open(&mut service, source);
    let (declaration, declaration_location) = occurrence(source, "café", 0);
    let (reference, reference_location) = occurrence(source, "caf&#233;", 0);
    assert_eq!(
        send(
            &mut service,
            request(
                2,
                "vize/nativeTemplateDefinition",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[declaration_location.clone()]}))
    );
    assert_eq!(
        send(
            &mut service,
            request(
                3,
                "vize/nativeTemplateReferences",
                declaration.0,
                declaration.1,
                true
            )
        ),
        Some(
            json!({"jsonrpc":"2.0","id":3,"result":[declaration_location,reference_location.clone()]})
        )
    );
    assert_eq!(
        send(
            &mut service,
            request(
                4,
                "vize/nativeTemplateReferences",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":4,"result":[reference_location]}))
    );
    assert_eq!(
        send(
            &mut service,
            request(5, "vize/nativeTemplateDefinition", 1, 1, false)
        ),
        Some(error(5, -32602, "Invalid native UTF-16 position"))
    );
}

#[test]
fn every_var_site_and_implicit_event_have_complete_array_response_shapes() {
    let mut service = service();
    let source = "<template><button @click='var value=$event;{var value=2;}return value;$event'/></template>";
    open(&mut service, source);
    let (_, first) = occurrence(source, "value", 0);
    let (_, second) = occurrence(source, "value", 1);
    let (reference, reference_location) = occurrence(source, "value", 2);
    let (event, first_event) = occurrence(source, "$event", 0);
    let (_, last_event) = occurrence(source, "$event", 1);
    assert_eq!(
        send(
            &mut service,
            request(
                2,
                "vize/nativeTemplateDefinition",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[first.clone(),second.clone()]}))
    );
    assert_eq!(
        send(
            &mut service,
            request(
                3,
                "vize/nativeTemplateReferences",
                reference.0,
                reference.1,
                true
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[first,second,reference_location]}))
    );
    assert_eq!(
        send(
            &mut service,
            request(4, "vize/nativeTemplateDefinition", event.0, event.1, false)
        ),
        Some(json!({"jsonrpc":"2.0","id":4,"result":[]}))
    );
    assert_eq!(
        send(
            &mut service,
            request(5, "vize/nativeTemplateReferences", event.0, event.1, true)
        ),
        Some(json!({"jsonrpc":"2.0","id":5,"result":[first_event,last_event]}))
    );
}

#[test]
fn actual_selected_whole_sfc_refusals_are_complete_and_do_not_replace_generic_navigation() {
    let mut service = service();
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    open(&mut service, source);
    let (_, declaration) = occurrence(source, "value", 0);
    let (reference, _) = occurrence(source, "value", 1);
    assert_eq!(
        send(
            &mut service,
            request(
                2,
                "vize/nativeTemplateDefinition",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(error(
            2,
            -32011,
            "Native original selected SFC observation refused"
        ))
    );
    assert_eq!(
        send(
            &mut service,
            request(3, "vize/nativeDefinition", reference.0, reference.1, false)
        ),
        Some(json!({"jsonrpc":"2.0","id":3,"result":declaration}))
    );
    open(
        &mut service,
        "<template><div v-for='caf&#233; in it&#101;ms'/></template>",
    );
    assert_eq!(
        send(
            &mut service,
            request(4, "vize/nativeTemplateDefinition", 0, 26, false)
        ),
        Some(error(
            4,
            -32011,
            "Native original selected SFC observation refused"
        ))
    );
}

#[test]
fn real_did_change_and_close_replace_or_retire_the_selected_owned_rpc_family() {
    let mut service = service();
    let source = "<template><button @click='let value=$event;return value'/></template>";
    open(&mut service, source);
    let (reference, declaration) = occurrence(source, "value", 0);
    assert_eq!(
        send(
            &mut service,
            request(
                2,
                "vize/nativeTemplateDefinition",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[declaration]}))
    );
    let changed = "<template><button @click='let fresh=$event;return fresh'/></template>";
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":URI,"version":2},"contentChanges":[{"text":changed}]}})
        ),
        None
    );
    let (reference, declaration) = occurrence(changed, "fresh", 0);
    assert_eq!(
        send(
            &mut service,
            request(
                3,
                "vize/nativeTemplateDefinition",
                reference.0,
                reference.1,
                false
            )
        ),
        Some(json!({"jsonrpc":"2.0","id":3,"result":[declaration]}))
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
            request(4, "vize/nativeTemplateDefinition", 0, 1, false)
        ),
        Some(error(4, -32602, "Native document unavailable"))
    );
}
