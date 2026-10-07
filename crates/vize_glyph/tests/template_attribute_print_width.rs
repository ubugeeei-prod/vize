//! Whole authored/reference bytes for directive attribute-prefix overflow (#7876).

use serde::Deserialize;
use std::{error::Error, fs, path::Path};
use vize_glyph::{FormatOptions, format_sfc, format_template};
use vize_l0::String;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    api: String,
    input: String,
    output: String,
    options: FormatOptions,
}

#[test]
fn original_and_boundary_outputs_are_complete_three_pass_fixed_points() -> Result<(), Box<dyn Error>>
{
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/corpus.json"
    ))?;
    assert_eq!(corpus.cases.len(), 16);
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/directive-print-width-7876",
    );
    for case in corpus.cases {
        let input = fs::read(directory.join(case.input.as_str()))?;
        let expected = fs::read(directory.join(case.output.as_str()))?;
        let mut previous = String::from(core::str::from_utf8(&input)?);
        for pass in 1..=3 {
            let output = match case.api.as_str() {
                "sfc" => {
                    let result = format_sfc(&previous, &case.options)?;
                    assert_eq!(
                        result.changed,
                        previous.as_bytes() != expected,
                        "{} pass {pass} changed flag",
                        case.id
                    );
                    result.code
                }
                "template" => format_template(&previous, &case.options)?,
                _ => panic!("unregistered fixture API"),
            };
            assert_eq!(
                output.as_bytes(),
                expected,
                "{} pass {pass} complete bytes",
                case.id
            );
            previous = output;
        }
    }
    Ok(())
}
