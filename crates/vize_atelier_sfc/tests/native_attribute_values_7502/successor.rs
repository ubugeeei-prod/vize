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
        "current":{"dom":"positive","ssr":"positive","vapor":"target-refusal"}
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
    Ok(())
}
