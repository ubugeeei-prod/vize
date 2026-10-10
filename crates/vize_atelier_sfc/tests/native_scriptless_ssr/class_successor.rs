use serde_json::{Value, json};
use vize_atelier_sfc::{NativeSsrSfcCompileOptions, compile_native_ssr_sfc};
use vize_l0::Allocator;

pub fn capture() -> Value {
    let source = "<template><div class='static'/></template>";
    let mut rows = Vec::new();
    for source_map in [true, false] {
        let arena = Allocator::default();
        let compilation = compile_native_ssr_sfc(
            &arena,
            source,
            NativeSsrSfcCompileOptions {
                source_map,
                ..Default::default()
            },
        );
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        let file = compilation
            .observation()
            .admitted()
            .unwrap()
            .into_template_view()
            .file()
            .unwrap();
        assert!(file.is_complete());
        let output = compilation.result().unwrap();
        rows.push(
            json!({"id":"class","source":source,"filename":"anonymous.vue","sourceMap":source_map,
            "code":output.code(),"mapText":output.source_map(),
            "map":output.source_map().map(|text|serde_json::from_str::<Value>(text).unwrap()),
            "links":output.document().links().iter().map(|link|json!({
                "authored":{"start":link.authored.start,"end":link.authored.end},
                "generated":{"start":link.generated.start,"end":link.generated.end},
                "name":link.name.as_deref(),"segment":link.segment
            })).collect::<Vec<_>>() }),
        );
    }
    assert_eq!(rows[0]["code"], rows[1]["code"]);
    json!({"baselineRevision":"815d9342ed252cad5802e58931b15166f25bf746", "historicalDisposition":"whole_sfc_refusal",
        "currentDisposition":"complete_original_sfc_module", "options":"unchanged NativeSsrSfcCompileOptions::default(); Recorded additionally sets source_map=true", "rows":rows})
}

pub fn unchanged(capture: &Value) {
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/native-scriptless-ssr-output.json"
    ))
    .unwrap();
    assert_eq!(capture["modules"], original["capture"]["modules"]);
    let previous = original["capture"]["refusals"].as_array().unwrap();
    let current = capture["refusals"].as_array().unwrap();
    assert_eq!(previous.len(), current.len() + 1);
    let mut index = 0;
    for row in previous {
        if row["id"] == "class" {
            for positive in capture["classSuccessor"]["rows"].as_array().unwrap() {
                assert_eq!(positive["id"], row["id"]);
                assert_eq!(positive["source"], row["source"]);
            }
        } else {
            assert_eq!(
                &current[index], row,
                "every unaffected whole refusal remains exact"
            );
            index += 1;
        }
    }
    assert_eq!(index, current.len());
}
