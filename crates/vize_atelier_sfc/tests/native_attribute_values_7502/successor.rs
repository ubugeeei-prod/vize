use super::{Test, check, inputs};
use serde_json::{Value, json};
use vize_l0::ToCompactString;

pub fn transition() -> Value {
    json!({
        "baseline":{
            "revision":"815d9342ed252cad5802e58931b15166f25bf746",
            "tree":"4c79ab4eea0b1f9f6eea62916de8ce9f9383dc99",
            "fixtureTree":"76783543e5d1dddb5a127b0f032d6f217909d847"
        },
        "originalReviewedOutputSha256":
            "d32b8138113cbae60ea28b82c5653fa9b5fdb40a8b9f8a90ebd7e67a88b9a2cb",
        "input":"original-regression-11",
        "historicalDisposition":"lower-refusal",
        "previousReviewedOutputSha256":
            "4802cc3a776f7711cbe32893858bf770407123a2745c34be9f35e9ed3b880428",
        "previousVersion":2,
        "changedOutcomes":[
            {"target":"vapor","linkMode":"Recorded","before":"target-refusal","after":"positive"},
            {"target":"vapor","linkMode":"NoLinks","before":"target-refusal","after":"positive"}
        ],
        "current":{"dom":"positive","ssr":"positive","vapor":"positive"}
    })
}

pub fn unchanged(packet: &Value) -> Test {
    let bytes = include_bytes!("../fixtures/native_attribute_values_7502/reviewed_output.json");
    check(
        inputs::sha256(bytes) == "d32b8138113cbae60ea28b82c5653fa9b5fdb40a8b9f8a90ebd7e67a88b9a2cb",
        "whole original reviewed bytes drift",
    )?;
    let archived: Value =
        serde_json::from_slice(bytes).map_err(|error| error.to_compact_string())?;
    let previous = archived["capture"]["rows"]
        .as_array()
        .ok_or("original rows")?;
    let current = packet["rows"].as_array().ok_or("current rows")?;
    check(
        current.len() == previous.len(),
        "whole original row population",
    )?;
    check(
        packet["transition"] == transition(),
        "exact narrow successor declaration",
    )?;
    for (row, original) in current.iter().zip(previous) {
        for key in ["id", "target", "linkMode"] {
            check(row[key] == original[key], "original outcome identity/order")?;
        }
        if row["id"] != "original-regression-11" {
            check(
                row == original,
                "every unaffected original whole row remains exact",
            )?;
        }
    }
    let bytes = include_bytes!("../fixtures/native_attribute_values_7502/reviewed_output_v2.json");
    check(
        inputs::sha256(bytes) == "4802cc3a776f7711cbe32893858bf770407123a2745c34be9f35e9ed3b880428",
        "whole reviewed V2 bytes drift",
    )?;
    let reviewed: Value =
        serde_json::from_slice(bytes).map_err(|error| error.to_compact_string())?;
    let previous = reviewed["capture"]["rows"]
        .as_array()
        .ok_or("whole V2 rows")?;
    check(current.len() == previous.len(), "whole V2 population")?;
    for (row, old) in current.iter().zip(previous) {
        if row["id"] == "original-regression-11" && row["target"] == "vapor" {
            let old = old.as_object().ok_or("whole prior class row")?;
            let current = row.as_object().ok_or("whole current class row")?;
            check(current.keys().eq(old.keys()), "exact changed-row fields")?;
            check(
                old["result"]["classification"] == "target-refusal",
                "named previous refusal",
            )?;
            for (key, value) in old.iter().filter(|(key, _)| key.as_str() != "result") {
                check(
                    current.get(key) == Some(value),
                    "whole changed-row custody stays exact",
                )?;
            }
        } else {
            check(row == old, "all 82 unaffected V2 whole rows stay exact")?;
        }
    }
    Ok(())
}
