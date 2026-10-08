//! A closed incoming script is part of the whole configured project answer.
use serde_json::json;

use super::{TestResult, check, fixture::Fixture, reads, runtime_enabled};

const SOURCE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/incoming/Source.ts.txt"
);
const CONSUMER: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/incoming/Consumer.ts.txt"
);

fn whole_project(fixture: &mut Fixture, query: &str, token: reads::Token) -> TestResult {
    let source = fixture.uri("src/Source.ts");
    let consumer = fixture.uri("src/Consumer.ts");
    let definition = reads::location(&source, (0, 20, 31));
    let uri = fixture.uri(query);
    let position = json!({"line":token.0,"character":token.1});
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
        definition.clone(),
    )?;
    for include in [true, false] {
        let expected = if include && query == "src/Source.ts" {
            json!([
                definition.clone(),
                reads::location(&consumer, (0, 9, 20)),
                reads::location(&consumer, (1, 22, 33))
            ])
        } else if include {
            // Preserve the native query-source-first group order verbatim.
            json!([
                reads::location(&consumer, (0, 9, 20)),
                reads::location(&consumer, (1, 22, 33)),
                definition.clone()
            ])
        } else {
            json!([reads::location(&consumer, (1, 22, 33))])
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
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn bare_exporter_references_retain_the_entire_closed_and_open_incoming_script() {
    if !runtime_enabled().unwrap() {
        return;
    }
    for newline in ["\n", "\r\n"] {
        let files = [
            ("src/Source.ts", SOURCE.replace('\n', newline)),
            ("src/Consumer.ts", CONSUMER.replace('\n', newline)),
        ];
        let mut fixture = Fixture::new(&files, false).unwrap();
        fixture.prove_native_diagnostics().unwrap();
        fixture.open_both(files[0].0, &files[0].1, "typescript", 1);
        // Consumer is on disk and in tsconfig, never opened before this answer.
        whole_project(&mut fixture, files[0].0, (0, 20, 31)).unwrap();
        fixture
            .refuse_writes(files[0].0, (0, 20, 31), "exportValueNext")
            .unwrap();
        fixture.open_both(files[1].0, &files[1].1, "typescript", 1);
        for (file, token) in [
            (files[0].0, (0, 20, 31)),
            (files[1].0, (0, 9, 20)),
            (files[1].0, (1, 22, 33)),
        ] {
            whole_project(&mut fixture, file, token).unwrap();
            fixture
                .refuse_writes(file, token, "exportValueNext")
                .unwrap();
        }
        fixture.reopen_both(files[0].0, "typescript", 1).unwrap();
        fixture.reopen_both(files[1].0, "typescript", 1).unwrap();
        whole_project(&mut fixture, files[0].0, (0, 20, 31)).unwrap();
        fixture.assert_disks().unwrap();
        fixture.finish().unwrap();
    }
}
