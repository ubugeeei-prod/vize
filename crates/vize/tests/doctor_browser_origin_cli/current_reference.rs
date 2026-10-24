//! One exact current reference; every original corpus byte remains authoritative.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt::Write as _, fs, path::Path};
use vize_l0::String;

fn hash(bytes: &[u8]) -> Result<String, std::fmt::Error> {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(output, "{byte:02x}")?;
    }
    Ok(output)
}

pub fn validate(directory: &Path, source: &Value) -> Result<Value, Box<dyn Error>> {
    let bytes = fs::read(directory.join("current-reference-7908.json"))?;
    assert_eq!(
        hash(&bytes)?,
        "f36cfb0cacc0058c44ae6ad5529bc25390f265dcb8796d601e1652603aee8b8d"
    );
    assert_eq!(
        hash(&fs::read(directory.join("source.json"))?)?,
        "dad987b8bbfd1d89640a4dc600da57545b038ab7006bede88453b91648439c26"
    );
    let authority: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(authority["schema"], "vize.browser-origin.current-reference");
    assert_eq!(authority["version"], 1);
    assert_eq!(
        authority["cases"]
            .as_array()
            .ok_or("authority cases must be an array")?
            .len(),
        1
    );
    let row = &authority["cases"][0];
    assert_eq!(row["path"], "PageTitle.vue");
    let case = source["cases"]
        .as_array()
        .ok_or("historical cases must be an array")?
        .iter()
        .find(|case| case["path"] == row["path"])
        .ok_or("PageTitle history must exist")?;
    assert_eq!(case["bytes"], 227);
    assert_eq!(case["sha256"], row["sha256"]);
    let input =
        fs::read(directory.join(case["file"].as_str().ok_or("history filename must exist")?))?;
    assert_eq!(input.len(), 227);
    assert_eq!(
        hash(&input)?.as_str(),
        row["sha256"].as_str().ok_or("input hash must exist")?
    );
    assert_eq!(row["historicalExpected"], case["expected"]);
    assert_eq!(row["historicalDoctorLocations"], case["doctorLocations"]);
    assert_eq!(
        row["currentExpected"],
        json!([{
            "file": "PageTitle.vue", "messages": [], "errorCount": 0, "warningCount": 0
        }])
    );
    assert_eq!(row["currentDoctorLocations"], json!([]));
    assert_ne!(row["currentExpected"], row["historicalExpected"]);
    assert_ne!(
        row["currentDoctorLocations"],
        row["historicalDoctorLocations"]
    );
    let law = &authority["historicProducerLaw"];
    let archived =
        fs::read(directory.join(law["file"].as_str().ok_or("law filename must exist")?))?;
    assert_eq!(
        hash(&archived)?.as_str(),
        law["sha256"].as_str().ok_or("law hash must exist")?
    );
    Ok(row.clone())
}
