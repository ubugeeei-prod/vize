//! Original inputs and independent whole references for nested directive calls.
use std::{error::Error, fs, path::Path};
use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn directive_calls_observe_actual_indentation_and_three_public_fixed_points()
-> Result<(), Box<dyn Error>> {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/corpus.json"
    ))?;
    let cases = corpus["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), 16);
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876",
    );
    for case in cases {
        let id = case["id"].as_str().ok_or("missing ID")?;
        let input = fs::read(directory.join(case["input"].as_str().ok_or("missing input")?))?;
        let expected = fs::read(directory.join(case["output"].as_str().ok_or("missing output")?))?;
        let options: FormatOptions = serde_json::from_value(case["options"].clone())?;
        let mut previous = std::string::String::from_utf8(input)?;
        for pass in 1..=3 {
            let result = format_sfc(&previous, &options)?;
            assert_eq!(
                result.changed,
                previous.as_bytes() != expected,
                "{id} pass{pass}: complete changed flag"
            );
            assert_eq!(
                result.code.as_bytes(),
                expected,
                "{id} pass{pass}: whole bytes"
            );
            previous = result.code.to_string();
        }
    }
    Ok(())
}
