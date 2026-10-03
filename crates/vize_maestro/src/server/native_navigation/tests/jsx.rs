mod lifecycle;

use super::{URI, definition, error, location, open, send, service};
use serde_json::{Value, json};

fn references_at(id: i32, line: u32, character: u32, include: bool) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeReferences","params":{"textDocument":{"uri":URI},"position":{"line":line,"character":character},"context":{"includeDeclaration":include}}})
}

#[test]
fn real_jsx_and_tsx_document_profiles_have_complete_original_utf16_responses() {
    let mut service = service();
    for language in ["javascriptreact", "typescriptreact"] {
        open(
            &mut service,
            "/*😀*/ import UI from 'dep';\r\nconst café=1;\r\nconst view=<UI.Button value={café}><div>{café}</div><UI.Button/></UI.Button>;",
            language,
            1,
        );
        assert_eq!(
            send(&mut service, definition(2, 2, 13)),
            Some(json!({"jsonrpc":"2.0","id":2,"result":location(0,14,16)}))
        );
        assert_eq!(
            send(&mut service, references_at(3, 2, 13, true)),
            Some(
                json!({"jsonrpc":"2.0","id":3,"result":[location(0,14,16),location(2,12,14),location(2,53,55)]})
            )
        );
        assert_eq!(
            send(&mut service, definition(4, 2, 30)),
            Some(json!({"jsonrpc":"2.0","id":4,"result":location(1,6,10)}))
        );
        assert_eq!(
            send(&mut service, references_at(5, 2, 42, false)),
            Some(json!({"jsonrpc":"2.0","id":5,"result":[location(2,29,33),location(2,41,45)]}))
        );
        for (id, character) in [(6, 16), (7, 23), (8, 37), (9, 67)] {
            assert_eq!(
                send(&mut service, definition(id, 2, character)),
                Some(json!({"jsonrpc":"2.0","id":id,"result":null}))
            );
        }
        assert_eq!(
            send(&mut service, definition(10, 0, 3)),
            Some(error(10, -32602, "Invalid native UTF-16 position"))
        );
    }
}

#[test]
fn jsx_native_refusals_are_whole_errors_without_legacy_or_extension_fallback() {
    let mut service = service();
    for language in ["javascript", "typescript"] {
        open(&mut service, "const UI=1;const view=<UI/>;", language, 1);
        assert_eq!(
            send(&mut service, definition(2, 0, 7)),
            Some(error(2, -32002, "Native original Program refused"))
        );
    }
    for language in ["javascriptreact", "typescriptreact"] {
        for source in [
            "const view=<Missing/>;",
            "const UI=1;const view=<this.Button/>;",
            "const view=<ns:tag/>;",
        ] {
            open(&mut service, source, language, 1);
            assert_eq!(
                send(&mut service, definition(3, 0, 7)),
                Some(error(3, -32003, "Native File observation refused"))
            );
            assert_eq!(
                send(&mut service, references_at(4, 0, 7, true)),
                Some(error(4, -32003, "Native File observation refused"))
            );
        }
        open(&mut service, "const view=<div>;", language, 1);
        assert_eq!(
            send(&mut service, definition(5, 0, 7)),
            Some(error(5, -32002, "Native original Program refused"))
        );
    }
    // Authentic TSX syntax does not grant semantic admission to unimplemented
    // TS variable annotations or JSX generic arguments in the current File.
    for source in [
        "const value:number=1;value;",
        "const UI=1;const view=<UI<Type>/>;",
    ] {
        open(&mut service, source, "typescriptreact", 1);
        assert_eq!(
            send(&mut service, definition(6, 0, 7)),
            Some(error(6, -32003, "Native File observation refused"))
        );
    }
}
