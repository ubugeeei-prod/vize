//! Full fixed oracle inventory and raw official semantic diagnostics.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(super) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

const MODES: [&str; 8] = [
    "template-function-no-prefix-map-false",
    "template-module-prefix-map-false",
    "template-real-bindings-function-no-prefix-map-false",
    "template-real-bindings-module-prefix-map-false",
    "template-function-no-prefix-map-true",
    "template-module-prefix-map-true",
    "template-real-bindings-function-no-prefix-map-true",
    "template-real-bindings-module-prefix-map-true",
];

fn resolve(value: &Value, document: &Value) -> Result<Value, String> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        let pointed = document
            .pointer(reference.strip_prefix('#').ok_or("reference")?)
            .ok_or("raw official reference target")?;
        return resolve(pointed, document);
    }
    if let Some(object) = value.as_object() {
        return object
            .iter()
            .map(|(key, value)| Ok((key.clone(), resolve(value, document)?)))
            .collect::<Result<serde_json::Map<_, _>, String>>()
            .map(Value::Object);
    }
    Ok(value.clone())
}

pub(super) fn byte_offset(source: &str, utf16: usize) -> Result<usize, String> {
    let mut units = 0;
    for (byte, character) in source.char_indices() {
        if units == utf16 {
            return Ok(byte);
        }
        units += character.len_utf16();
    }
    if units == utf16 {
        Ok(source.len())
    } else {
        Err("official UTF-16 offset boundary".into())
    }
}

fn semantic(errors: &Value, document: &Value) -> Result<Vec<Value>, String> {
    errors
        .as_array()
        .ok_or("complete ordered raw errors")?
        .iter()
        .map(|e| {
            Ok(json!({"code":e["code"],"message":e["message"],
            "location":resolve(&e["loc"],document)?}))
        })
        .collect()
}

pub(super) fn authenticate(
    template: &str,
    provenance: &Value,
    official: &Value,
    expected: &[Value],
) -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/n8n-if-key-regression/official");
    let mut inventory = vec!["record.json".to_string(), "template.txt".to_string()];
    for mode in MODES {
        inventory.push(format!("{mode}.json"));
        inventory.push(format!(
            "{mode}.{}",
            if mode.contains("real-bindings") {
                "ts"
            } else {
                "js"
            }
        ));
    }
    inventory.sort();
    let files = provenance["oracleFiles"]
        .as_object()
        .ok_or("oracle files")?;
    let declared = files.keys().cloned().collect::<Vec<_>>();
    let mut actual = std::fs::read_dir(&root)
        .map_err(|e| e.to_string())?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    actual.sort();
    if actual != inventory || declared != inventory {
        return Err("complete literal official packet inventory changed".into());
    }
    for (path, hash) in files {
        let bytes = std::fs::read(root.join(path)).map_err(|e| e.to_string())?;
        if hash != &json!(sha256(&bytes)) {
            return Err(format!("complete official oracle changed: {path}"));
        }
    }
    let modes = official["record"]["templates"]
        .as_array()
        .ok_or("official modes")?;
    if modes.len() != MODES.len() {
        return Err("exact eight official modes required".into());
    }
    let mut records = vec![Value::Null; 10];
    *records.get_mut(9).ok_or("fixed official original index")? = official["record"].clone();
    let record_document = json!({"records":records});
    for (mode, name) in modes.iter().zip(MODES) {
        let raw: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("{name}.json"))).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if mode["name"] != name || mode["status"] != "returned" || raw["status"] != "returned" {
            return Err(format!("official mode order/status changed: {name}"));
        }
        let errors = semantic(&raw["result"]["errors"], &raw)?;
        if semantic(&mode["errors"], &record_document)? != errors {
            return Err("complete raw and record diagnostic lengths differ".into());
        }
        let module = std::fs::read_to_string(root.join(format!(
            "{name}.{}",
            if name.contains("real-bindings") {
                "ts"
            } else {
                "js"
            }
        )))
        .map_err(|e| e.to_string())?;
        if raw["result"]["code"] != module || raw["result"]["source"] != template {
            return Err("complete official code/source packet differs".into());
        }
        let want = if name.contains("function-no-prefix") {
            expected.len()
        } else {
            0
        };
        if errors.len() != want {
            return Err(format!("official mode diagnostics changed: {name}"));
        }
        // Maps-on includes an independently retained official shared-location
        // remap defect; exact raw packets are authenticated above, never repaired.
        if !name.ends_with("map-false") {
            continue;
        }
        for (error, want) in errors.iter().zip(expected) {
            if error["code"] != want["officialCode"]
                || error["message"] != want["officialMessage"]
                || error["location"] != want["officialLocation"]
            {
                return Err(format!(
                    "ordered official semantic diagnostic differs: {error}"
                ));
            }
        }
    }
    Ok(())
}
