//! Whole syntax-only continuation outputs retain authored text boundaries.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    input: String,
    output: String,
    input_sha256: String,
    output_sha256: String,
    options: FormatOptions,
}

fn sha256(bytes: &[u8]) -> vize_l0::String {
    let mut hex = vize_l0::String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        hex.push_str(vize_l0::cstr!("{byte:02x}").as_str());
    }
    hex
}

#[test]
fn complete_original_and_prefix_controls_keep_three_fixed_points() -> Result<(), Box<dyn Error>> {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/corpus.json"
    ))?;
    assert_eq!(corpus.cases.len(), 11);
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876",
    );
    for case in corpus.cases {
        let input = fs::read(directory.join(case.input.as_str()))?;
        let expected = fs::read(directory.join(case.output.as_str()))?;
        assert_eq!(sha256(&input).as_str(), case.input_sha256.as_str());
        assert_eq!(sha256(&expected).as_str(), case.output_sha256.as_str());
        let mut previous = String::from(core::str::from_utf8(&input)?);
        for pass in 1..=3 {
            let result = format_sfc(&previous, &case.options)?;
            assert_eq!(
                result.changed,
                previous.as_bytes() != expected,
                "{} pass {pass}",
                case.id
            );
            assert_eq!(
                result.code.as_bytes(),
                expected,
                "{} pass {pass} whole bytes",
                case.id
            );
            previous = result.code;
        }
    }
    Ok(())
}
