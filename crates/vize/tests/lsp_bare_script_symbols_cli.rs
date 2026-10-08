#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "independent stdio fixtures")]
#![expect(clippy::disallowed_methods, reason = "independent stdio fixtures")]
#![expect(clippy::disallowed_macros, reason = "independent stdio fixtures")]

#[path = "lsp_bare_script_symbols_cli/controls.rs"]
mod controls;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "lsp_bare_script_symbols_cli/fixture.rs"]
mod fixture;
#[path = "lsp_bare_script_symbols_cli/incoming.rs"]
mod incoming;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "lsp_bare_script_symbols_cli/reads.rs"]
mod reads;
#[path = "lsp_bare_script_symbols_cli/session.rs"]
mod session;

use fixture::Fixture;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn runtime_enabled() -> TestResult<bool> {
    let resolved = corsa_requirement::required_or_skip::<std::path::PathBuf>(None);
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return Ok(false);
    }
    resolved.ok_or("bare stdio laws require the actual native runtime")?;
    Ok(true)
}

fn check(actual: serde_json::Value, expected: serde_json::Value) -> TestResult {
    if actual != expected {
        return Err(
            format!("whole response mismatch: actual={actual:#}; expected={expected:#}").into(),
        );
    }
    Ok(())
}

const PLAIN: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/H/plain.ts.txt"
);
const ORDINARY: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/H/ordinary.d.ts.txt"
);
const AUTHORED_DOM: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/H/lib.dom.d.ts.txt"
);

#[test]
fn historical_h_inputs_keep_the_three_complete_original_sha256_without_a_runtime() {
    use sha2::{Digest, Sha256};

    for (source, expected) in [
        (
            PLAIN,
            "76d1d63a212f6d1dd90dcebc7b7e06f432c5d4e9f19f0f7f6fa2be65ce958879",
        ),
        (
            ORDINARY,
            "24356e674a61b86f5296a552fc132fe7e95fcbee35a5c1b4e31a0f0a812a2051",
        ),
        (
            AUTHORED_DOM,
            "482ef0c05cc06e4e84910734b9aa986c8902a59611c75664f890faf5d9df2279",
        ),
    ] {
        let actual = Sha256::digest(source.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn historical_original_bare_read_vectors_refuse_writes_and_reopen_unchanged_disks() {
    if !runtime_enabled().unwrap() {
        return;
    }
    for newline in ["\n", "\r\n"] {
        let sources = [
            ("src/plain.ts", PLAIN.replace('\n', newline)),
            ("src/ordinary.d.ts", ORDINARY.replace('\n', newline)),
            ("src/lib.dom.d.ts", AUTHORED_DOM.replace('\n', newline)),
        ];
        let mut fixture = Fixture::new(&sources, false).unwrap();
        fixture.prove_native_diagnostics().unwrap();
        for (file, declaration, usage, replacement) in [
            ("src/plain.ts", (0, 13, 31), (1, 27, 45), "authoredValue"),
            (
                "src/ordinary.d.ts",
                (0, 21, 34),
                (1, 44, 57),
                "authoredOrdinaryValue",
            ),
            (
                "src/lib.dom.d.ts",
                (3, 21, 34),
                (4, 41, 54),
                "authoredDeclaredValue",
            ),
        ] {
            let source = sources
                .iter()
                .find(|(name, _)| *name == file)
                .unwrap()
                .1
                .clone();
            fixture.open_both(file, &source, "typescript", 1);
            reads::identity(&mut fixture, file, declaration, usage, true).unwrap();
            fixture
                .refuse_writes(file, declaration, replacement)
                .unwrap();
            // Both a new line and an astral character before the first token
            // change authored positions while the physical file stays exact.
            let dirty = format!("// 😀{newline}/*😀*/ {source}");
            fixture.change_both(file, &dirty, 2);
            let shifted = |(line, start, end)| {
                (
                    line + 1,
                    start + u32::from(line == 0) * 7,
                    end + u32::from(line == 0) * 7,
                )
            };
            reads::identity(
                &mut fixture,
                file,
                shifted(declaration),
                shifted(usage),
                true,
            )
            .unwrap();
            fixture
                .refuse_writes(file, shifted(declaration), replacement)
                .unwrap();
            // Closing the dirty buffer discards its overlay; reopening reads
            // the untouched original disk with the same client version.
            fixture.reopen_both(file, "typescript", 1).unwrap();
            reads::identity(&mut fixture, file, declaration, usage, true).unwrap();
            fixture
                .refuse_writes(file, declaration, replacement)
                .unwrap();
            fixture.assert_disks().unwrap();
        }
        fixture.finish().unwrap();
    }
}

#[test]
fn javascript_utf16_identity_excludes_same_named_shadows_strings_and_comments() {
    if !runtime_enabled().unwrap() {
        return;
    }
    let source = include_str!(
        "../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/Unicode.js.txt"
    );
    for newline in ["\n", "\r\n"] {
        let sources = [("src/Unicode.js", source.replace('\n', newline))];
        let mut fixture = Fixture::new(&sources, false).unwrap();
        fixture.prove_native_diagnostics().unwrap();
        fixture.open_both(sources[0].0, &sources[0].1, "javascript", 1);
        reads::identity(&mut fixture, sources[0].0, (0, 20, 24), (1, 22, 26), true).unwrap();
        reads::identity(&mut fixture, sources[0].0, (2, 23, 27), (2, 38, 42), true).unwrap();
        fixture
            .refuse_writes(sources[0].0, (0, 20, 24), "caféNext")
            .unwrap();
        let uri = fixture.uri(sources[0].0);
        for position in [
            json!({"line":0,"character":3}),
            json!({"line":99,"character":0}),
        ] {
            for method in ["textDocument/definition", "textDocument/references"] {
                assert_eq!(
                    fixture
                        .vize
                        .request(
                            method,
                            &uri,
                            json!({"position":position,"context":{"includeDeclaration":true}})
                        )
                        .unwrap(),
                    serde_json::Value::Null
                );
            }
        }
        fixture.reopen_both(sources[0].0, "javascript", 1).unwrap();
        reads::identity(&mut fixture, sources[0].0, (0, 20, 24), (1, 22, 26), true).unwrap();
        fixture.assert_disks().unwrap();
        fixture.finish().unwrap();
    }
}
