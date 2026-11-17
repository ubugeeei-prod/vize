//! Typed projection of the actual producer's canonical α contract JSON.
//! A roundtrip check rejects unsupported/unknown/missing fields while the
//! returned value keeps every original string, null and type dependency.

use super::super::error::HostError;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};
use vize_croquis::croquis::alpha::{
    ComponentContract, EmitContract, PropContract, ReactivityContract, SignatureContract,
    SlotContract,
};
use vize_l2::summary::{AlphaEntry, AlphaPages, Facet};

pub(super) fn project(
    pages: &AlphaPages,
    names: &[String],
) -> Result<Map<String, Value>, HostError> {
    let mut output = Map::new();
    for facet in Facet::ALL {
        if !names.iter().any(|name| name == facet.group()) {
            continue;
        }
        let rows = match facet {
            Facet::Signature => vec![json!([
                pages.signature.name.as_str(),
                contract::<SignatureContract>(&pages.signature.params)?
            ])],
            Facet::Prop => rows::<PropContract>(&pages.props)?,
            Facet::Emit => rows::<EmitContract>(&pages.emits)?,
            Facet::Slot => rows::<SlotContract>(&pages.slots)?,
            Facet::Reactivity => rows::<ReactivityContract>(&pages.reactivity)?,
            Facet::Component => rows::<ComponentContract>(&pages.components)?,
        };
        output.insert(facet.group().to_owned(), Value::Array(rows));
    }
    Ok(output)
}

fn rows<T: DeserializeOwned + Serialize>(entries: &[AlphaEntry]) -> Result<Vec<Value>, HostError> {
    entries
        .iter()
        .map(|entry| {
            Ok(json!([
                entry.name.as_str(),
                contract::<T>(&entry.contract)?
            ]))
        })
        .collect()
}

fn contract<T: DeserializeOwned + Serialize>(text: &str) -> Result<Value, HostError> {
    let typed: T = serde_json::from_str(text)
        .map_err(|error| HostError::Split(format!("invalid production alpha contract: {error}")))?;
    let original: Value = serde_json::from_str(text)
        .map_err(|error| HostError::Split(format!("invalid production alpha JSON: {error}")))?;
    let roundtrip = serde_json::to_value(typed).map_err(|error| {
        HostError::Split(format!("invalid production alpha serialization: {error}"))
    })?;
    if roundtrip != original {
        return Err(HostError::Split(
            "production alpha contract contains unknown or missing fields".into(),
        ));
    }
    Ok(original)
}
