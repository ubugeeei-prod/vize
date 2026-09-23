//! JS plugin SDK spike — the napi host (Davinci P4-16, charter #29).
//!
//! The decided shape (record: `docs/davinci/plan/phase-4-records/p4-16.md`):
//!
//! - **Serialized visit batches, not proxies.** Per plugin per document the
//!   host builds one JSON batch of the S2 page (only the node kinds the
//!   manifest visits, the dense parent array, only the demanded facts) and
//!   makes one call. The proxy arm ([`PluginDocumentHandle`]) is kept only
//!   as the measured alternative: every property read is a napi crossing.
//! - **Sync napi on the JS thread, not a worker.** A JS function can only be
//!   called on its own thread; the Rust rules keep their rayon parallelism
//!   and the JS batches run after them, outside the fused walks.
//! - **Static demands in the manifest.** `demands: ["templateScopes"]` is
//!   resolved against the fact registry before anything runs; the batch
//!   carries only declared groups.
//! - **Cost attribution and content keys.** Every plugin's time is in the
//!   lint output; every result has an S0 content key and a P5-1b manifest
//!   covering the plugin's version, code, visit and demand sets, and file.
//!
//! P5-13 adds cross-process disk reuse. GA (all four hook families and the
//! `@vizejs/plugin-sdk` package) is P6-7.

#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod batch;
mod document;
mod error;
mod facts;
mod plugin_cache;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::Instant;

use napi::Env;
use napi::bindgen_prelude::{Error, FunctionRef, Result, Status};
use napi_derive::napi;
use vize_davinci::fact::FactManager;

use batch::{PluginDiagnostic, PluginSpec, build_batch, diagnostics, sort, validate_spec};
use document::PluginDocument;
use error::HostError;
use facts::{REGISTRY, TemplateScopes, resolve_demands};
use plugin_cache::{PluginCacheInput, cache, content_key, validate_cache_inputs};

/// A plugin as the SDK's `definePlugin` hands it to the host.
#[napi(object, object_to_js = false)]
pub struct JsPluginNapi {
    pub name: String,
    pub version: String,
    /// The SDK's digest of the plugin's rule sources.
    pub fingerprint: String,
    pub visit: Option<Vec<String>>,
    pub demands: Option<Vec<String>>,
    /// Configuration and ambient values read by the rule. Required for caching.
    pub cache_inputs: Option<Vec<PluginCacheInputNapi>>,
    /// `(batchJson) => reportsJson` — one call per document.
    pub run: FunctionRef<String, String>,
}

/// One stable value a cached JS plugin declares it reads outside the batch.
#[napi(object)]
pub struct PluginCacheInputNapi {
    pub name: String,
    pub value: String,
}

#[napi(object)]
#[derive(Default)]
pub struct PluginLintOptionsNapi {
    pub filename: Option<String>,
    /// Reuse a result whose content key this process has already seen.
    pub cache: Option<bool>,
    /// Optional directory for reusing results across Node processes.
    pub cache_dir: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct PluginDiagnosticNapi {
    pub rule_id: String,
    pub plugin: String,
    pub severity: String,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// One plugin's cost in this lint run.
#[napi(object)]
pub struct PluginCostNapi {
    pub name: String,
    pub version: String,
    pub content_key: String,
    /// Nodes in the plugin's batch (0 when served from the cache).
    pub nodes: u32,
    pub batch_bytes: u32,
    pub reports: u32,
    pub cached: bool,
    /// Host time for this plugin: batch build, the JS call, report mapping.
    pub elapsed_ns: f64,
    /// The JS call alone.
    pub js_ns: f64,
}

#[napi(object)]
pub struct PluginLintOutputNapi {
    pub filename: String,
    pub diagnostics: Vec<PluginDiagnosticNapi>,
    pub plugins: Vec<PluginCostNapi>,
}

fn host_error(error: HostError) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}

fn to_napi(diagnostic: PluginDiagnostic) -> PluginDiagnosticNapi {
    PluginDiagnosticNapi {
        rule_id: diagnostic.rule_id,
        plugin: diagnostic.plugin,
        severity: "warning".to_owned(),
        message: diagnostic.message,
        start: diagnostic.start,
        end: diagnostic.end,
        line: diagnostic.line,
        column: diagnostic.column,
        end_line: diagnostic.end_line,
        end_column: diagnostic.end_column,
    }
}

/// Lint one SFC with JS plugins: one batch and one call per plugin, every
/// plugin's cost attributed, diagnostics in the host's one order.
#[napi(js_name = "lintWithPlugins")]
pub fn lint_with_plugins(
    env: Env,
    source: String,
    plugins: Vec<JsPluginNapi>,
    options: Option<PluginLintOptionsNapi>,
) -> Result<PluginLintOutputNapi> {
    let options = options.unwrap_or_default();
    let filename = options
        .filename
        .unwrap_or_else(|| "anonymous.vue".to_owned());
    let use_cache = options.cache == Some(true);
    let cache_dir = options.cache_dir.as_deref().map(Path::new);
    let document = PluginDocument::build(&source, &filename).map_err(host_error)?;
    let mut manager = FactManager::new(&REGISTRY);
    let mut all = Vec::new();
    let mut costs = Vec::with_capacity(plugins.len());
    for plugin in &plugins {
        let started = Instant::now();
        let demands = plugin.demands.clone().unwrap_or_default();
        let spec = PluginSpec {
            name: &plugin.name,
            version: &plugin.version,
            fingerprint: &plugin.fingerprint,
            visit: plugin.visit.as_deref(),
            demands: &demands,
        };
        validate_spec(&spec).map_err(host_error)?;
        let cache_inputs: Vec<PluginCacheInput<'_>> = plugin
            .cache_inputs
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|input| PluginCacheInput {
                name: &input.name,
                value: &input.value,
            })
            .collect();
        if use_cache {
            validate_cache_inputs(&plugin.name, plugin.cache_inputs.is_some(), &cache_inputs)
                .map_err(host_error)?;
        }
        let key = content_key(&source, &filename, &spec, &cache_inputs);
        let hit = use_cache
            .then(|| cache().lock().ok()?.get(&key, cache_dir))
            .flatten();
        let (found, nodes, bytes, js_ns, cached) = match hit {
            Some(found) => (found, 0, 0, 0.0, true),
            None => {
                let built = build_batch(&document, &spec, &mut manager).map_err(host_error)?;
                let bytes = built.json.len() as u32;
                let run = plugin.run.borrow_back(&env)?;
                let called = Instant::now();
                let reports = run.call(built.json)?;
                let js_ns = called.elapsed().as_nanos() as f64;
                let found = diagnostics(&document, &plugin.name, &reports).map_err(host_error)?;
                if use_cache && let Ok(mut map) = cache().lock() {
                    map.put(&key, found.clone(), cache_dir);
                }
                (found, built.nodes, bytes, js_ns, false)
            }
        };
        costs.push(PluginCostNapi {
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            content_key: key,
            nodes,
            batch_bytes: bytes,
            reports: found.len() as u32,
            cached,
            elapsed_ns: started.elapsed().as_nanos() as f64,
            js_ns,
        });
        all.extend(found);
    }
    sort(&mut all);
    Ok(PluginLintOutputNapi {
        filename,
        diagnostics: all.into_iter().map(to_napi).collect(),
        plugins: costs,
    })
}

/// The proxy arm the spike measured and rejected: the same document behind
/// a handle whose every read is one napi call.
#[napi]
pub struct PluginDocumentHandle {
    document: PluginDocument,
    scopes: Vec<(u32, Vec<facts::ScopeEntry>)>,
}

/// Open the proxy-arm handle over one SFC, computing `templateScopes`.
#[napi(js_name = "openPluginDocument")]
pub fn open_plugin_document(
    source: String,
    filename: Option<String>,
) -> Result<PluginDocumentHandle> {
    let filename = filename.unwrap_or_else(|| "anonymous.vue".to_owned());
    let document = PluginDocument::build(&source, &filename).map_err(host_error)?;
    let demand = resolve_demands("proxy", &["templateScopes".to_owned()]).map_err(host_error)?;
    let mut manager = FactManager::new(&REGISTRY);
    manager
        .compute(&document, demand)
        .map_err(|error| Error::new(Status::GenericFailure, format!("{error:?}")))?;
    let view = manager.view::<facts::JsPluginHost>();
    let scopes = view
        .get::<TemplateScopes>()
        .map(|table| {
            table
                .iter()
                .map(|(id, entries)| (*id, entries.clone()))
                .collect()
        })
        .unwrap_or_default();
    Ok(PluginDocumentHandle { document, scopes })
}

#[napi]
impl PluginDocumentHandle {
    /// How many nodes the document has (ids are `0..count`).
    #[napi]
    pub fn count(&self) -> u32 {
        self.document.nodes.len() as u32
    }

    #[napi]
    pub fn kind(&self, id: u32) -> Option<String> {
        self.node(id).map(|node| node.kind.to_owned())
    }

    /// The owning node, or -1.
    #[napi]
    pub fn parent(&self, id: u32) -> i32 {
        self.node(id)
            .and_then(|node| node.parent)
            .map_or(-1, |parent| parent as i32)
    }

    /// `name`, `value`, `alias.value`, `alias.key` or `alias.index`.
    #[napi]
    pub fn field(&self, id: u32, key: String) -> Option<String> {
        let node = self.node(id)?;
        let alias = node.alias.as_ref();
        match key.as_str() {
            "name" => node.name.clone(),
            "value" => node.value.clone(),
            "alias.value" => alias.map(|alias| alias.value.clone()),
            "alias.key" => alias.and_then(|alias| alias.key.clone()),
            "alias.index" => alias.and_then(|alias| alias.index.clone()),
            _ => None,
        }
    }

    /// The names scope `id` binds (`templateScopes`), or `null`.
    #[napi]
    pub fn scope(&self, id: u32) -> Option<Vec<ScopeEntryNapi>> {
        let (_, entries) = self.scopes.iter().find(|(scope, _)| *scope == id)?;
        let own = |entry: &facts::ScopeEntry| ScopeEntryNapi {
            name: entry.name.clone(),
            position: entry.position.to_owned(),
        };
        Some(entries.iter().map(own).collect())
    }
}

/// One `templateScopes` entry on the proxy arm.
#[napi(object)]
pub struct ScopeEntryNapi {
    pub name: String,
    pub position: String,
}

impl PluginDocumentHandle {
    fn node(&self, id: u32) -> Option<&document::PluginNode> {
        self.document.nodes.get(id as usize)
    }
}
