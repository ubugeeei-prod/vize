use super::{Test, check};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use vize_l0::{String, ToCompactString, cstr};

pub const PACK_SHA256: &str = "130e3b247f1333f66ed1a529b7618e8de28cd926b33e6a57e2a12de640645b8f";
pub const LEDGER_SHA256: &str = "a7f0d7ea54e8d687057e36a778d8d7fab8bfc2f5a9fd5e31a3c226b3bc5928f8";
pub const FILENAME: &str = "AttributeValues7502.vue";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Inputs {
    pub schema: String,
    pub version: u32,
    pub ledger_sha256: String,
    pub original_fix: String,
    pub original_parent: String,
    pub test_blob: String,
    pub reporter_blob: String,
    pub authority: String,
    pub fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    pub source: String,
    pub source_sha256: String,
    pub existing_expected_template_html: String,
    pub expected_template_html_sha256: String,
    pub existing_options: String,
    pub disposition: String,
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect()
}

pub fn load() -> Test<Inputs> {
    let bytes = include_bytes!("../fixtures/native_attribute_values_7502/original_inputs.json");
    check(sha256(bytes) == PACK_SHA256, "original input pack bytes")?;
    let inputs: Inputs =
        serde_json::from_slice(bytes).map_err(|error| error.to_compact_string())?;
    check(
        inputs.schema == "vize.native-attribute-values-7502.inputs"
            && inputs.version == 1
            && inputs.ledger_sha256 == LEDGER_SHA256
            && inputs.original_fix == "578f770216b7b3b5b8c168698e664b85addfd6e0"
            && inputs.original_parent == "a2fc9c9ec4fb2dd913dfce146691caa8009b3976"
            && inputs.test_blob == "70872e2d54bc06021c4e791bb61a99538071f909"
            && inputs.reporter_blob == "36819dc73df85b56b389bda9e4d10bfb9511d05b"
            && inputs.authority
                == "Verbatim original sources and legacy HTML assertions; no native code/map expectations"
            && inputs.fixtures.len() == 14,
        "original authority pins",
    )?;
    for (index, fixture) in inputs.fixtures.iter().enumerate() {
        let id = if index == 0 {
            String::from("reporter-7502")
        } else {
            cstr!("original-regression-{index}")
        };
        let disposition = if index == 11 || index == 12 {
            "lower-refusal"
        } else {
            "positive"
        };
        check(
            fixture.id == id
                && sha256(fixture.source.as_bytes()) == fixture.source_sha256
                && sha256(fixture.existing_expected_template_html.as_bytes())
                    == fixture.expected_template_html_sha256
                && fixture.existing_options == "compile_vapor Default::default()"
                && fixture.disposition == disposition,
            "verbatim original fixture identity/options/disposition",
        )?;
    }
    Ok(inputs)
}
