//! Namespaced JS fact providers: static closure, audited values and cache stamps.
#![expect(
    clippy::disallowed_types,
    reason = "plugin wire values use std strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "plugin wire values use std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "plugin wire values use std strings"
)]

mod cache;
mod catalog;
mod output;
#[cfg(feature = "napi")]
mod runtime;
#[cfg(test)]
mod tests;

use core::fmt;
use serde_json::{Map, Value};
use vize_l0::fact::FactManager;

use super::batch::{BuiltBatch, PluginSpec, build_batch};
use super::document::PluginDocument;
use super::error::HostError;
use super::facts::JS_VISIBLE;
use super::plugin_cache::PluginCacheInput;

pub use catalog::ProviderCatalog;
pub use output::ProviderOutput;
#[cfg(feature = "napi")]
pub use runtime::{JsFactProviderNapi, ProviderCostNapi, ProviderHost};

/// A provider's static declaration, inspected before any JS call.
pub struct ProviderSpec<'a> {
    pub plugin: PluginSpec<'a>,
    pub provides: &'a [String],
    pub inputs_declared: bool,
    pub inputs: &'a [PluginCacheInput<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError(pub String);

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<HostError> for ProviderError {
    fn from(error: HostError) -> Self {
        Self(error.to_string())
    }
}

pub fn refusal(plugin: &str, detail: &str) -> ProviderError {
    ProviderError(format!("{plugin}: invalid fact provider ({detail})"))
}

/// Build only the declared native and custom groups into one visit batch.
pub fn custom_batch(
    document: &PluginDocument,
    spec: &PluginSpec<'_>,
    manager: &mut FactManager<'_, PluginDocument>,
    custom: &Map<String, Value>,
) -> Result<BuiltBatch, ProviderError> {
    let native: Vec<_> = spec
        .demands
        .iter()
        .filter(|name| JS_VISIBLE.contains(&name.as_str()))
        .cloned()
        .collect();
    let mut built = build_batch(
        document,
        &PluginSpec {
            demands: &native,
            ..*spec
        },
        manager,
    )?;
    if let Some(name) = spec
        .demands
        .iter()
        .find(|name| !JS_VISIBLE.contains(&name.as_str()) && !custom.contains_key(*name))
    {
        return Err(refusal(
            spec.name,
            &format!("fact group `{name}` was not provided"),
        ));
    }
    if custom.is_empty() {
        return Ok(built);
    }
    let mut batch: Value = serde_json::from_str(&built.json)
        .map_err(|error| refusal(spec.name, &error.to_string()))?;
    let facts = batch
        .get_mut("facts")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| refusal(spec.name, "host batch lacks its fact object"))?;
    for name in spec.demands {
        if let Some(value) = custom.get(name) {
            facts.insert(name.clone(), value.clone());
        } else if !JS_VISIBLE.contains(&name.as_str()) {
            return Err(refusal(
                spec.name,
                &format!("fact group `{name}` was not provided"),
            ));
        }
    }
    built.json =
        serde_json::to_string(&batch).map_err(|error| refusal(spec.name, &error.to_string()))?;
    Ok(built)
}
