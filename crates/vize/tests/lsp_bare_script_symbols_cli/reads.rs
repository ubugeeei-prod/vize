//! Literal authored ranges are independent of the implementation's replies.
use serde_json::{Value, json};

use super::fixture::Fixture;
use super::{TestResult, check};

pub(super) type Token = (u32, u32, u32);

pub(super) fn range((line, start, end): Token) -> Value {
    json!({"start":{"line":line,"character":start},"end":{"line":line,"character":end}})
}

pub(super) fn location(uri: &str, token: Token) -> Value {
    json!({"uri":uri,"range":range(token)})
}

pub(super) fn identity(
    fixture: &mut Fixture,
    file: &str,
    declaration: Token,
    usage: Token,
    enabled: bool,
) -> TestResult {
    let uri = fixture.uri(file);
    let definition = location(&uri, declaration);
    for token in [declaration, usage] {
        let position = json!({"line":token.0,"character":token.1});
        // Stock native uses the original physical document; Vize's public
        // single-definition contract uses a scalar location.
        check(
            fixture.native.request(
                "textDocument/definition",
                &uri,
                json!({"position":position}),
            )?,
            json!([definition.clone()]),
        )?;
        check(
            fixture.vize.request(
                "textDocument/definition",
                &uri,
                json!({"position":position}),
            )?,
            if enabled {
                definition.clone()
            } else {
                Value::Null
            },
        )?;
        for include in [true, false] {
            let expected = if include {
                json!([definition.clone(), location(&uri, usage)])
            } else {
                json!([location(&uri, usage)])
            };
            let params = json!({"position":position,"context":{"includeDeclaration":include}});
            check(
                fixture
                    .native
                    .request("textDocument/references", &uri, params.clone())?,
                expected.clone(),
            )?;
            check(
                fixture
                    .vize
                    .request("textDocument/references", &uri, params)?,
                if enabled { expected } else { Value::Null },
            )?;
        }
    }
    Ok(())
}
