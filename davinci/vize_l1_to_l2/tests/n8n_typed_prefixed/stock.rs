//! Independent stock output and actual bare-JS parse failures stay explicit.
use super::{contract, packets::digest};
use serde_json::{Value, json};

const STOCK: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/prefixed-official-language-controls.json"
);

#[test]
fn pinned_stock_languages_preserve_whole_outputs_and_unresolved_js_failures() {
    assert_eq!(
        digest(STOCK.as_bytes()),
        "ebcb8a01dbd9bb8470b221b0e1fd98d487b92a6716a5d27fd36c20d29bcf11d3"
    );
    let stock: Value =
        serde_json::from_str(STOCK).expect("complete independent stock language packets");
    assert_eq!(stock["compiler"]["version"], "3.5.26");
    assert_eq!(
        stock["compiler"]["sha256"],
        "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec"
    );
    let originals = stock["originals"].as_array().expect("whole originals");
    assert_eq!(originals.len(), 2);
    for original in originals {
        let name = original["name"].as_str().expect("authored original name");
        let source = original["source"].as_str().expect("whole original bytes");
        assert_eq!(digest(source.as_bytes()), contract(name)["sourceSha256"]);
    }
    let records = stock["records"]
        .as_array()
        .expect("exact complete packet inventory");
    assert_eq!(records.len(), 8);
    let inventory = records
        .iter()
        .map(|row| {
            json!({"name":row["name"],
        "mode":row["options"]["compilerOptions"]["mode"],
        "isTS":row["options"]["compilerOptions"]["isTS"]})
        })
        .collect::<Vec<_>>();
    assert_eq!(
        inventory,
        vec![
            json!({"name":"n8n_original_instance_ai","mode":"function","isTS":false}),
            json!({"name":"n8n_original_instance_ai","mode":"function","isTS":true}),
            json!({"name":"n8n_original_instance_ai","mode":"module","isTS":false}),
            json!({"name":"n8n_original_instance_ai","mode":"module","isTS":true}),
            json!({"name":"n8n-FormInput","mode":"function","isTS":false}),
            json!({"name":"n8n-FormInput","mode":"function","isTS":true}),
            json!({"name":"n8n-FormInput","mode":"module","isTS":false}),
            json!({"name":"n8n-FormInput","mode":"module","isTS":true}),
        ]
    );
    for row in records {
        assert_eq!(row["output"]["errors"], json!([]));
        let code = row["output"]["code"]
            .as_str()
            .expect("whole stock generated output");
        assert_eq!(digest(code.as_bytes()), row["codeSha256"]);
        assert_eq!(row["bareJavaScriptCheck"]["status"], 1);
        assert_eq!(row["bareJavaScriptCheck"]["signal"], Value::Null);
        assert_eq!(row["bareJavaScriptCheck"]["error"], Value::Null);
    }
}
