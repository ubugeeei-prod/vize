//! Whole historical inputs and observed OXC width conversion boundaries.
#![expect(
    clippy::disallowed_types,
    reason = "complete formatter observation packets"
)]

use std::{fs, path::Path};

use oxc_formatter_core::LineWidth;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use vize_glyph::{FormatOptions, format_script, format_sfc};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Corpus {
    schema: String,
    issue: u32,
    source_base: String,
    phase: String,
    original_call_input: String,
    widths: Vec<u32>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    target: String,
    input: String,
    input_sha256: String,
    default_expected: String,
    default_expected_sha256: String,
}

fn hash(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_pinned(root: &Path, file: &str, expected: &str) -> String {
    let bytes = fs::read(root.join(file)).unwrap();
    assert_eq!(hash(&bytes), expected, "{file}");
    String::from_utf8(bytes).unwrap()
}

#[test]
fn whole_historical_outputs_and_all_width_boundaries_are_retained_before_verdicts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/print-width-conversion-6098/cases.json"
    )).unwrap();
    assert_eq!(corpus.schema, "vize.glyph.printWidthConversion.v1");
    assert_eq!(corpus.issue, 6098);
    assert_eq!(
        corpus.source_base,
        "fbb2f58a596621adff00e8e3cf7aac2e35b70160"
    );
    assert_eq!(corpus.phase, "before-observation-only");
    assert_eq!(corpus.cases.len(), 4);
    assert_eq!(corpus.widths.len(), 12);
    let original = fs::read(root.join(&corpus.original_call_input)).unwrap();
    let call = corpus
        .cases
        .iter()
        .find(|case| case.id == "indented-direct-call")
        .unwrap();
    assert_eq!(fs::read(root.join(&call.input)).unwrap(), original);

    let default_width = FormatOptions::default()
        .to_oxc_format_options()
        .line_width
        .value();
    eprintln!(
        "formatter-width-engine:{}",
        serde_json::json!({
            "default": default_width,
            "u16Boundaries": [0u16, 1, 40, 80, 100, 120, 65535].map(|width| {
                serde_json::json!({"input":width,"outcome":LineWidth::try_from(width)
                    .map(|value|value.value()).map_err(|error|error.to_string())})
            })
        })
    );

    let mut observed = Vec::new();
    for case in &corpus.cases {
        let input = read_pinned(&root, &case.input, &case.input_sha256);
        let expected = read_pinned(&root, &case.default_expected, &case.default_expected_sha256);
        for &width in &corpus.widths {
            let options = FormatOptions {
                print_width: width,
                ..FormatOptions::default()
            };
            let mut source = input.clone();
            let mut passes = Vec::new();
            for pass in 0..3 {
                let outcome = if case.target == "script" {
                    format_script(&source, &options).map(|code| (code.to_string(), None))
                } else {
                    assert_eq!(case.target, "sfc");
                    format_sfc(&source, &options)
                        .map(|result| (result.code.to_string(), Some(result.changed)))
                };
                let packet = match outcome {
                    Ok((code, changed)) => {
                        let packet = serde_json::json!({"pass":pass,"input":source,"code":code,"changed":changed,"error":null});
                        source = code;
                        packet
                    }
                    Err(error) => {
                        serde_json::json!({"pass":pass,"input":source,"code":null,"changed":null,"error":error.to_string()})
                    }
                };
                passes.push(packet);
            }
            let packet = serde_json::json!({
                "id":case.id,"target":case.target,"input":input,"inputSha256":case.input_sha256,
                "options":options,"engineWidth":options.to_oxc_format_options().line_width.value(),
                "defaultExpected":expected,"passes":passes
            });
            eprintln!("formatter-width-api:{}", packet);
            observed.push(packet);
        }
    }
    assert_eq!(observed.len(), 48);
    for packet in &observed {
        let passes = packet["passes"].as_array().unwrap();
        let [first, second, third] = passes.as_slice() else {
            panic!("three complete passes")
        };
        for pass in passes {
            assert!(pass["error"].is_null(), "{packet}");
        }
        assert_eq!(first["code"], second["code"], "{packet}");
        assert_eq!(second["code"], third["code"], "{packet}");
        if packet["options"]["printWidth"] == 100 {
            assert_eq!(first["code"], packet["defaultExpected"], "{packet}");
        }
    }
}
