//! Authored entity DATA must survive HTML → JS formatting → HTML consumption.
//! These are legacy public formatter laws, not native whole-adapter admission.
#![expect(clippy::unwrap_used, reason = "pinned test data asserts by panicking")]
use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn complete_encoded_ampersand_outputs_retain_data_and_reach_fixed_points() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-encoded-ampersand-data-6098/cases.json"
    ))
    .unwrap();
    let rows = corpus["cases"].as_array().unwrap();
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/formatter/sfc-encoded-ampersand-data-6098");
    let observations: Vec<_> = rows
        .iter()
        .map(|row| {
            let id = row["id"].as_str().unwrap();
            let source =
                std::fs::read_to_string(directory.join(row["file"].as_str().unwrap())).unwrap();
            // The closed CLI adapter requires App.vue. Consume the canonical
            // registered bytes here too; keep the original corpus file frozen.
            let source = if id == "bare-sequence-reference" {
                let registered = std::fs::read_to_string(directory.join("App.vue.txt")).unwrap();
                assert_eq!(registered, source);
                registered
            } else {
                source
            };
            let first = format_sfc(&source, &FormatOptions::default()).unwrap();
            let second = format_sfc(&first.code, &FormatOptions::default()).unwrap();
            let third = format_sfc(&second.code, &FormatOptions::default()).unwrap();
            println!(
                "glyph-ampersand-rust {}",
                serde_json::json!({
                    "id": id, "source": source, "first": first.code,
                    "second": second.code, "third": third.code
                })
            );
            (id, source, first.code, second.code, third.code)
        })
        .collect();
    assert_eq!(observations.len(), 13);
    for (id, source, first, second, third) in observations {
        assert_eq!(first, second, "{id}: first full output is stable");
        assert_eq!(second, third, "{id}: third full output is stable");
        if id == "bare-sequence-reference" {
            assert_eq!(
                first.as_str(),
                include_str!(
                    "../../../tests/_fixtures/differential/formatter/sfc-encoded-ampersand-data-6098/reference.expected.txt"
                ),
                "the registered canonical input retains its complete genuine baseline output"
            );
        }
        if matches!(
            id,
            "quoted-reference"
                | "decimal-reference"
                | "hex-reference"
                | "amp-reference"
                | "call-reference"
                | "grouped-sequence-reference"
        ) {
            let body = source
                .strip_prefix("<template>")
                .unwrap()
                .strip_suffix("</template>\n")
                .unwrap();
            let expected = format!("<template>\n  {body}\n</template>\n");
            assert_eq!(
                first.as_str(),
                expected,
                "{id}: protect authored DATA on normal formatting path"
            );
        }
    }
}
