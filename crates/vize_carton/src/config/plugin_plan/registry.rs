//! The capability registry, read from the same JSON file the JS resolver imports.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::{FxHashMap, String};

const REGISTRY_JSON: &str = include_str!("../loader/plugin-registry.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginRegistry {
    pub plan_schema: u32,
    pub api_versions: Vec<u32>,
    pub products: Vec<String>,
    pub backends: Vec<String>,
    pub admitted_backends: Vec<String>,
    pub capabilities: FxHashMap<String, RegisteredCapability>,
    pub lint_documents: Vec<String>,
    pub record_kinds: Vec<String>,
    pub fact_groups: Vec<String>,
    pub reserved_directives: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RegisteredCapability {
    pub schema_versions: Vec<u32>,
    pub api_versions: Vec<u32>,
    /// Products that must implement the capability or refuse the plan.
    pub consumers: Vec<String>,
    pub callbacks: Vec<String>,
    /// Products that implement it today.
    pub admitted_by: Vec<String>,
}

/// The registry compiled into this build; an unreadable registry is a build defect.
pub fn plugin_registry() -> &'static PluginRegistry {
    static REGISTRY: OnceLock<PluginRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        serde_json::from_str(REGISTRY_JSON).unwrap_or_else(|_| PluginRegistry::refusing())
    })
}

impl PluginRegistry {
    /// Admits nothing, so every non-empty plan fails closed.
    fn refusing() -> Self {
        Self {
            plan_schema: 0,
            api_versions: Vec::new(),
            products: Vec::new(),
            backends: Vec::new(),
            admitted_backends: Vec::new(),
            capabilities: FxHashMap::default(),
            lint_documents: Vec::new(),
            record_kinds: Vec::new(),
            fact_groups: Vec::new(),
            reserved_directives: Vec::new(),
        }
    }
}
