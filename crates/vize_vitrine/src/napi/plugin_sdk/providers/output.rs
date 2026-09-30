//! Accept exactly declared fact tables and compare canonical audited results.
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use vize_l0::key::source_block_key;

use super::{ProviderError, refusal};

const MAX_RESULT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct ProviderOutput {
    pub values: Map<String, Value>,
    pub result_key: String,
}

impl ProviderOutput {
    pub fn audit(
        plugin: &str,
        provides: &[String],
        first: &str,
        second: &str,
    ) -> Result<Self, ProviderError> {
        let first = Self::parse(plugin, provides, first)?;
        let second = Self::parse(plugin, provides, second)?;
        if first != second {
            return Err(refusal(
                plugin,
                "same-input runs produced different fact tables",
            ));
        }
        let canonical =
            serde_json::to_string(&first).map_err(|error| refusal(plugin, &error.to_string()))?;
        let result_key = source_block_key("plugin-fact-result", &[], &canonical).to_string();
        Ok(Self {
            values: first,
            result_key,
        })
    }

    fn parse(
        plugin: &str,
        provides: &[String],
        json: &str,
    ) -> Result<Map<String, Value>, ProviderError> {
        if json.len() > MAX_RESULT_BYTES {
            return Err(refusal(plugin, "result exceeds the 4 MiB limit"));
        }
        let values: Map<String, Value> = serde_json::from_str(json).map_err(|error| {
            refusal(
                plugin,
                &format!("result must be a JSON fact object ({error})"),
            )
        })?;
        if values.len() != provides.len() || provides.iter().any(|name| !values.contains_key(name))
        {
            return Err(refusal(plugin, "result keys must exactly match provides"));
        }
        for (name, value) in &values {
            let rows = value
                .as_array()
                .ok_or_else(|| refusal(plugin, &format!("`{name}` must be a fact table array")))?;
            let mut keys = BTreeSet::new();
            for row in rows {
                let Some([key, _]) = row.as_array().map(Vec::as_slice) else {
                    return Err(refusal(
                        plugin,
                        &format!("`{name}` rows must be [key, value] pairs"),
                    ));
                };
                if !(key.is_string() || key.as_u64().is_some_and(|key| u32::try_from(key).is_ok()))
                {
                    return Err(refusal(
                        plugin,
                        &format!("`{name}` keys must be strings or u32 node IDs"),
                    ));
                }
                if !keys.insert(key.to_string()) {
                    return Err(refusal(
                        plugin,
                        &format!("`{name}` contains a duplicate key"),
                    ));
                }
            }
        }
        Ok(values)
    }
}
