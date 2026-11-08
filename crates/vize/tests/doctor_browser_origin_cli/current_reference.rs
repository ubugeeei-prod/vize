//! Exact current references; every original corpus byte remains authoritative.
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

const HISTORIC_CORPUS_SHA256: &str =
    "dad987b8bbfd1d89640a4dc600da57545b038ab7006bede88453b91648439c26";

fn validate_authority(
    directory: &Path,
    source: &Value,
    file: &str,
    expected_sha256: &str,
) -> Result<Value, Box<dyn Error>> {
    let bytes = fs::read(directory.join(file))?;
    assert_eq!(hash(&bytes)?, expected_sha256, "{file} must be byte-exact");
    let authority: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(authority["schema"], "vize.browser-origin.current-reference");
    assert_eq!(authority["version"], 1);
    assert_eq!(authority["historicCorpusSha256"], HISTORIC_CORPUS_SHA256);
    assert_eq!(
        authority["cases"]
            .as_array()
            .ok_or("authority cases must be an array")?
            .len(),
        1
    );
    let row = &authority["cases"][0];
    let case = source["cases"]
        .as_array()
        .ok_or("historical cases must be an array")?
        .iter()
        .find(|case| case["path"] == row["path"])
        .ok_or("current reference history must exist")?;
    assert_eq!(case["bytes"], row["bytes"]);
    assert_eq!(case["sha256"], row["sha256"]);
    let input =
        fs::read(directory.join(case["file"].as_str().ok_or("history filename must exist")?))?;
    assert_eq!(
        u64::try_from(input.len())?,
        row["bytes"].as_u64().ok_or("input byte count must exist")?
    );
    assert_eq!(
        hash(&input)?.as_str(),
        row["sha256"].as_str().ok_or("input hash must exist")?
    );
    assert_eq!(row["historicalExpected"], case["expected"]);
    assert_eq!(row["historicalDoctorLocations"], case["doctorLocations"]);
    assert!(
        row["currentExpected"] != row["historicalExpected"]
            || row["currentDoctorLocations"] != row["historicalDoctorLocations"],
        "{} must record a current behavior change",
        row["path"]
    );
    if let Some(law) = authority.get("historicProducerLaw") {
        let archived =
            fs::read(directory.join(law["file"].as_str().ok_or("law filename must exist")?))?;
        assert_eq!(
            hash(&archived)?.as_str(),
            law["sha256"].as_str().ok_or("law hash must exist")?
        );
    }
    Ok(row.clone())
}

pub fn validate(directory: &Path, source: &Value) -> Result<Value, Box<dyn Error>> {
    assert_eq!(
        hash(&fs::read(directory.join("source.json"))?)?,
        HISTORIC_CORPUS_SHA256
    );
    let watch = validate_authority(
        directory,
        source,
        "current-reference-7908.json",
        "f36cfb0cacc0058c44ae6ad5529bc25390f265dcb8796d601e1652603aee8b8d",
    )?;
    let unicode_columns = validate_authority(
        directory,
        source,
        "current-reference-12069.json",
        "5b575ad51651ea3830ebc424d4bdd20f8c1648d16a49e11aff988822406a06a1",
    )?;
    assert_eq!(watch["path"], "PageTitle.vue");
    assert_eq!(
        watch["currentExpected"],
        json!([{
            "file": "PageTitle.vue", "messages": [], "errorCount": 0, "warningCount": 0
        }])
    );
    assert_eq!(watch["currentDoctorLocations"], json!([]));
    assert_eq!(unicode_columns["path"], "UnicodeCrlf.vue");
    assert_eq!(
        unicode_columns["currentExpected"][0]["messages"][0]["column"],
        38
    );
    assert_eq!(
        unicode_columns["currentExpected"][0]["messages"][1]["column"],
        31
    );
    Ok(json!({
        "PageTitle.vue": watch,
        "UnicodeCrlf.vue": unicode_columns,
    }))
}
