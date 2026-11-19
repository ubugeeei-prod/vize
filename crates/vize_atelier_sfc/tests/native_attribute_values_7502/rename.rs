//! V4 of the reviewed output is V3 with the L1 component owners renamed.
//!
//! `rename_v4.json` names each old and new type name. Every gate that compares
//! the current capture with V1, V2 or V3 first spells the capture back with
//! [`previous`], so those gates stay exact; [`verify`] proves the V4 bytes are
//! V3's modulo that rename and nothing else.
use super::{Test, check, inputs};
use serde_json::{Map, Value};
use vize_l0::ToCompactString;

const RENAME: &[u8] = include_bytes!("../fixtures/native_attribute_values_7502/rename_v4.json");
const V3: &[u8] =
    include_bytes!("../fixtures/native_attribute_values_7502/reviewed_output_v3.json");
const V4: &[u8] =
    include_bytes!("../fixtures/native_attribute_values_7502/reviewed_output_v4.json");

/// `(old, new)` pairs, longest new name first so no name shadows another.
fn names() -> Result<Vec<(std::string::String, std::string::String)>, vize_l0::String> {
    check(
        inputs::sha256(RENAME)
            == "0a275499386391d1bbfee25077c7bb3516b3c817c1f630348afc90684761132f",
        "rename record drift",
    )?;
    let record: Value =
        serde_json::from_slice(RENAME).map_err(|error| error.to_compact_string())?;
    check(
        record.as_object().is_some_and(|object| object.len() == 4)
            && record["schema"] == "vize.native-attribute-values-7502.rename"
            && record["from"] == 3
            && record["to"] == 4,
        "rename record shape",
    )?;
    let mut pairs: Vec<_> = record["names"]
        .as_object()
        .ok_or("rename names")?
        .iter()
        .map(|(old, new)| Ok((old.clone(), new.as_str().ok_or("rename target")?.to_owned())))
        .collect::<Result<_, &str>>()?;
    pairs.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.1.cmp(&b.1)));
    Ok(pairs)
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Replace whole-word `from` with `to`.
fn replace_word(text: &str, from: &str, to: &str) -> std::string::String {
    let bytes = text.as_bytes();
    let mut out = std::string::String::with_capacity(text.len());
    let mut last = 0;
    for (at, _) in text.match_indices(from) {
        let end = at + from.len();
        let bounded =
            (at == 0 || !is_word(bytes[at - 1])) && (end == bytes.len() || !is_word(bytes[end]));
        if bounded && at >= last {
            out.push_str(&text[last..at]);
            out.push_str(to);
            last = end;
        }
    }
    out.push_str(&text[last..]);
    out
}

fn spell(value: &Value, pairs: &[(std::string::String, std::string::String)]) -> Value {
    let text = |text: &str| {
        pairs.iter().fold(text.to_owned(), |text, (old, new)| {
            replace_word(&text, new, old)
        })
    };
    match value {
        Value::String(value) => Value::String(text(value.as_str())),
        Value::Array(items) => Value::Array(items.iter().map(|item| spell(item, pairs)).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (text(key.as_str()), spell(value, pairs)))
                .collect::<Map<_, _>>(),
        ),
        other => other.clone(),
    }
}

/// The capture spelled with the names V1–V3 recorded.
pub fn previous(capture: &Value) -> Result<Value, vize_l0::String> {
    Ok(spell(capture, &names()?))
}

fn mentions(bytes: &[u8], name: &str) -> bool {
    bytes
        .windows(name.len())
        .any(|window| window == name.as_bytes())
}

/// V4 equals V3 once the new names are spelled back, and differs from it only there.
pub fn verify() -> Test {
    check(
        inputs::sha256(V3) == "1e313a90dbfc89003e63f8fed5aadc17dc686d5b73275b5f8ae3dabed83e93fd",
        "whole reviewed V3 bytes drift",
    )?;
    check(
        inputs::sha256(V4) == "a298e1c14da44fa7f893a73793a7c0e7fa1b6250140e51633f3fd7bfe2a977c4",
        "whole reviewed V4 bytes drift",
    )?;
    let pairs = names()?;
    // Spelling back is unambiguous only if V3 never used a new name.
    for (old, new) in &pairs {
        check(!mentions(V3, new), "V3 already spells a new name")?;
        check(mentions(V3, old), "V3 never spells a renamed name")?;
        check(!mentions(V4, old), "V4 still spells an old name")?;
    }
    let parse = |bytes: &[u8]| -> Result<Value, vize_l0::String> {
        serde_json::from_slice(bytes).map_err(|error| error.to_compact_string())
    };
    let (v3, v4) = (parse(V3)?, parse(V4)?);
    for key in ["schema", "state"] {
        check(v4[key] == v3[key], "V4 keeps V3's schema and state")?;
    }
    check(
        v3["version"] == 3 && v4["version"] == 4,
        "V4 succeeds V3 by one version",
    )?;
    check(
        v4.as_object().map(Map::len) == v3.as_object().map(Map::len),
        "V4 adds no field",
    )?;
    check(v4["capture"] != v3["capture"], "V4 records the new names")?;
    check(
        spell(&v4["capture"], &pairs) == v3["capture"],
        "V4 capture equals V3 modulo the rename",
    )
}

#[test]
fn word_replacement_respects_identifier_boundaries() {
    assert_eq!(
        replace_word(
            "OwnedComponent { x: OwnedComponentX, y: _OwnedComponent }",
            "OwnedComponent",
            "N"
        ),
        "N { x: OwnedComponentX, y: _OwnedComponent }"
    );
}
