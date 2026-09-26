//! JS plugin SDK spike — the napi host (Davinci P4-16, charter #29).
//!
//! The decided shape (record: `docs/davinci/plan/phase-4-records/p4-16.md`):
//!
//! - **Serialized visit batches, not proxies.** Per plugin per document the
//!   host builds one JSON batch of the L2 page (only the node kinds the
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
//!   lint output; every result has an L0 content key and a P5-1b manifest
//!   covering the plugin's version, code, visit and demand sets, and file.
//!
//! P5-13 adds cross-process disk reuse. GA (all four hook families and the
//! `@vizejs/plugin-sdk` package) is P6-7.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "N-API values cross the boundary as std `String`s"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "N-API values cross the boundary as std `String`s"
)]

mod batch;
mod document;
mod error;
mod facts;
mod plugin_cache;
mod production;
mod providers;
mod proxy;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::Instant;

use napi::Env;
use napi::bindgen_prelude::{Error, FunctionRef, Result, Status};
use napi_derive::napi;
use vize_davinci::fact::FactManager;

use batch::{
    PluginDiagnostic, PluginSpec, build_batch, diagnostics, sort, valid_cached, validate_spec,
};
use document::PluginDocument;
use error::HostError;
use facts::REGISTRY;
use plugin_cache::{PluginCacheInput, cache, content_key, validate_cache_inputs};
use providers::{JsFactProviderNapi, ProviderCostNapi, ProviderHost};

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

#[napi(object, object_to_js = false)]
#[derive(Default)]
pub struct PluginLintOptionsNapi {
    pub filename: Option<String>,
    /// Reuse a result whose content key this process has already seen.
    pub cache: Option<bool>,
    /// Optional directory for reusing results across Node processes.
    pub cache_dir: Option<String>,
    /// Run identical batches twice and reject different diagnostics or fixes.
    /// Always enabled for cache misses.
    pub validate_determinism: Option<bool>,
    /// Namespaced providers resolved from each plugin's static fact demand.
    pub fact_providers: Option<Vec<JsFactProviderNapi>>,
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
    /// JS executions (two for an audited miss, zero for a hit).
    pub executions: u32,
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
    /// Suggested replacements over host-owned node spans.
    pub fixes: Vec<PluginFixNapi>,
    pub fact_providers: Vec<ProviderCostNapi>,
}

/// One plugin autofix, confined to the reported node's source span.
#[napi(object)]
pub struct PluginFixNapi {
    pub rule_id: String,
    pub plugin: String,
    pub start: u32,
    pub end: u32,
    pub text: String,
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
    let provider_definitions = options.fact_providers.unwrap_or_default();
    let has_providers = !provider_definitions.is_empty();
    let mut providers = ProviderHost::new(provider_definitions)?;
    let filename = options
        .filename
        .unwrap_or_else(|| "anonymous.vue".to_owned());
    let use_cache = options.cache == Some(true);
    let audit = use_cache || options.validate_determinism == Some(true);
    let cache_dir = options.cache_dir.as_deref().map(Path::new);
    providers.configure_cache(use_cache, cache_dir);
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
        if !has_providers {
            validate_spec(&spec).map_err(host_error)?;
        }
        let mut cache_inputs: Vec<PluginCacheInput<'_>> = plugin
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
        let provided = if has_providers {
            Some(providers.batch(&env, &document, &spec, &mut manager)?)
        } else {
            None
        };
        if let Some(provided) = &provided {
            cache_inputs.extend(
                provided
                    .cache_inputs
                    .iter()
                    .map(|(name, value)| PluginCacheInput { name, value }),
            );
        }
        let key = content_key(&source, &filename, &spec, &cache_inputs);
        let cache_key = key.as_deref().filter(|_| use_cache);
        let hit = cache_key
            .and_then(|key| cache().lock().ok()?.get(key, cache_dir))
            .filter(|found| valid_cached(&document, &plugin.name, found));
        let (found, nodes, bytes, js_ns, cached) = match hit {
            Some(found) => (found, 0, 0, 0.0, true),
            None => {
                let built = match provided {
                    Some(provided) => provided.built,
                    None => build_batch(&document, &spec, &mut manager).map_err(host_error)?,
                };
                let bytes = built.json.len() as u32;
                let run = plugin.run.borrow_back(&env)?;
                let called = Instant::now();
                let reports = run.call(built.json.clone())?;
                let mut js_ns = called.elapsed().as_nanos() as f64;
                let found = diagnostics(&document, &plugin.name, &reports).map_err(host_error)?;
                if audit {
                    let called = Instant::now();
                    let repeated = run.call(built.json)?;
                    js_ns += called.elapsed().as_nanos() as f64;
                    let repeated =
                        diagnostics(&document, &plugin.name, &repeated).map_err(host_error)?;
                    if found != repeated {
                        return Err(host_error(HostError::Nondeterministic {
                            plugin: plugin.name.clone(),
                        }));
                    }
                }
                if let Some(key) = cache_key
                    && let Ok(mut map) = cache().lock()
                {
                    map.put(key, found.clone(), cache_dir);
                }
                (found, built.nodes, bytes, js_ns, false)
            }
        };
        costs.push(PluginCostNapi {
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            content_key: key.unwrap_or_default(),
            nodes,
            batch_bytes: bytes,
            reports: found.len() as u32,
            cached,
            executions: if cached {
                0
            } else if audit {
                2
            } else {
                1
            },
            elapsed_ns: started.elapsed().as_nanos() as f64,
            js_ns,
        });
        all.extend(found);
    }
    sort(&mut all);
    let fixes = all
        .iter()
        .filter_map(|diagnostic| {
            diagnostic.fix.as_ref().map(|text| PluginFixNapi {
                rule_id: diagnostic.rule_id.clone(),
                plugin: diagnostic.plugin.clone(),
                start: diagnostic.start,
                end: diagnostic.end,
                text: text.clone(),
            })
        })
        .collect();
    Ok(PluginLintOutputNapi {
        filename,
        diagnostics: all.into_iter().map(to_napi).collect(),
        plugins: costs,
        fixes,
        fact_providers: providers.costs(),
    })
}
